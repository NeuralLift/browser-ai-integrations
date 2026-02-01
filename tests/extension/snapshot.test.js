const fs = require('fs');
const path = require('path');

// Mock Chrome API
global.chrome = {
  runtime: {
    onMessage: {
      addListener: jest.fn(),
    },
  },
};

// Mock window.scrollTo
global.scrollTo = jest.fn();

// Load content.js
const contentJsPath = path.resolve(__dirname, '../../extension/content.js');
const contentJsCode = fs.readFileSync(contentJsPath, 'utf8');

// Use eval to run the script in the test context
eval(contentJsCode);

describe('Snapshot Generation', () => {
  beforeEach(() => {
    document.body.innerHTML = '';
    jest.clearAllMocks();

    // Mock getBoundingClientRect
    window.Element.prototype.getBoundingClientRect = function () {
      return {
        width: parseFloat(this.style.width) || 100,
        height: parseFloat(this.style.height) || 30,
        top: parseFloat(this.style.top) || 0,
        left: parseFloat(this.style.left) || 0,
        bottom:
          (parseFloat(this.style.top) || 0) +
          (parseFloat(this.style.height) || 30),
        right:
          (parseFloat(this.style.left) || 0) +
          (parseFloat(this.style.width) || 100),
      };
    };

    // Mock getComputedStyle
    window.getComputedStyle = (el) => {
      return {
        display: el.style.display || 'block',
        visibility: el.style.visibility || 'visible',
        opacity: el.style.opacity || '1',
        cursor: el.style.cursor || 'auto',
      };
    };

    // Set viewport size
    window.innerWidth = 1024;
    window.innerHeight = 768;
  });

  test('should include interactive elements', () => {
    document.body.innerHTML = `
      <button id="btn1">Click me</button>
      <a href="https://example.com" id="link1">Link</a>
      <input type="text" id="input1" placeholder="Enter text">
    `;

    const snapshot = generateSnapshot();
    expect(snapshot.tree).toHaveLength(3);

    expect(snapshot.tree[0]).toMatchObject({
      role: 'button',
      name: 'Click me',
      tag: 'BUTTON',
    });

    expect(snapshot.tree[1]).toMatchObject({
      role: 'link',
      name: 'Link',
      tag: 'A',
    });

    expect(snapshot.tree[2]).toMatchObject({
      role: 'textbox',
      name: 'Enter text',
      tag: 'INPUT',
    });
  });

  test('should assign sequential ref IDs', () => {
    document.body.innerHTML = `
      <button>1</button>
      <button>2</button>
      <button>3</button>
    `;

    const snapshot = generateSnapshot();
    expect(snapshot.tree[0].id).toBe(1);
    expect(snapshot.tree[1].id).toBe(2);
    expect(snapshot.tree[2].id).toBe(3);
  });

  test('should exclude non-interactive elements', () => {
    document.body.innerHTML = `
      <div>Just a div</div>
      <span>Just a span</span>
      <p>Just a paragraph</p>
      <button>Interactive</button>
    `;

    const snapshot = generateSnapshot();
    expect(snapshot.tree).toHaveLength(1);
    expect(snapshot.tree[0].name).toBe('Interactive');
  });

  test('should exclude hidden elements', () => {
    document.body.innerHTML = `
      <button style="display: none;">Hidden</button>
      <button style="visibility: hidden;">Invisible</button>
      <div style="display: none;">
        <button>Inside hidden div</button>
      </div>
      <button>Visible</button>
    `;

    const snapshot = generateSnapshot();
    expect(snapshot.tree).toHaveLength(1);
    expect(snapshot.tree[0].name).toBe('Visible');
  });

  test('should exclude elements out of viewport', () => {
    document.body.innerHTML = `
      <button style="top: -100px; height: 50px;">Off screen top</button>
      <button style="top: 800px; height: 50px;">Off screen bottom</button>
      <button style="top: 100px; height: 50px;">In viewport</button>
    `;

    const snapshot = generateSnapshot();
    expect(snapshot.tree).toHaveLength(1);
    expect(snapshot.tree[0].name).toBe('In viewport');
  });

  test('should exclude elements with data-browser-agent-ui', () => {
    document.body.innerHTML = `
      <button data-browser-agent-ui="true">Agent UI Button</button>
      <button>Regular Button</button>
    `;

    const snapshot = generateSnapshot();
    expect(snapshot.tree).toHaveLength(1);
    expect(snapshot.tree[0].name).toBe('Regular Button');
  });

  test('should calculate correct bounds', () => {
    document.body.innerHTML = `
      <button style="top: 50px; left: 100px; width: 200px; height: 40px;">Bouncy</button>
    `;

    const snapshot = generateSnapshot();
    expect(snapshot.tree[0].bounds).toEqual({
      x: 100,
      y: 50,
      width: 200,
      height: 40,
    });
  });

  test('should handle aria-label and other accessible names', () => {
    document.body.innerHTML = `
      <button aria-label="Close Dialog">X</button>
      <input type="text" title="Username">
      <img src="test.png" alt="Profile Picture" role="button">
    `;

    const snapshot = generateSnapshot();
    expect(snapshot.tree[0].name).toBe('Close Dialog');
    expect(snapshot.tree[1].name).toBe('Username');
    expect(snapshot.tree[2].name).toBe('Profile Picture');
  });
});
