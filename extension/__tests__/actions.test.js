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

// Mock window.onNavigate for testing navigate_to
global.onNavigate = jest.fn();

// Load content.js and hack it for navigate_to testing
const contentJsPath = path.resolve(__dirname, '../content.js');
let contentJsCode = fs.readFileSync(contentJsPath, 'utf8');
contentJsCode = contentJsCode.replace(
  'window.location.assign(command.url)',
  'onNavigate(command.url)'
);

// Use eval to run the script in the test context
eval(contentJsCode);

describe('Action Executor', () => {
  beforeEach(() => {
    document.body.innerHTML = '';
    jest.clearAllMocks();

    // Mock getBoundingClientRect
    window.Element.prototype.getBoundingClientRect = function () {
      return {
        width: 100,
        height: 30,
        top: 0,
        left: 0,
        bottom: 30,
        right: 100,
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

  test('navigate_to should call onNavigate (hacked window.location.assign)', () => {
    const command = { type: 'navigate_to', url: 'https://example.com' };
    const result = executeAction(command);

    expect(result.success).toBe(true);
    expect(global.onNavigate).toHaveBeenCalledWith('https://example.com');
  });

  test('click_element should trigger click on element found by ref', () => {
    document.body.innerHTML = '<button id="target">Click Me</button>';
    const button = document.getElementById('target');
    const clickSpy = jest.spyOn(button, 'click');

    // Generate snapshot to populate refToElementMap
    generateSnapshot();

    const command = { type: 'click_element', ref: 1 };
    const result = executeAction(command);

    expect(result.success).toBe(true);
    expect(clickSpy).toHaveBeenCalled();
  });

  test('type_text should set value and dispatch events', () => {
    document.body.innerHTML = '<input type="text" id="target">';
    const input = document.getElementById('target');
    const inputEventSpy = jest.fn();
    const changeEventSpy = jest.fn();

    input.addEventListener('input', inputEventSpy);
    input.addEventListener('change', changeEventSpy);

    // Generate snapshot to populate refToElementMap
    generateSnapshot();

    const command = { type: 'type_text', ref: 1, text: 'Hello' };
    const result = executeAction(command);

    expect(result.success).toBe(true);
    expect(input.value).toBe('Hello');
    expect(inputEventSpy).toHaveBeenCalled();
    expect(changeEventSpy).toHaveBeenCalled();
  });

  test('scroll_to should call window.scrollTo', () => {
    const command = { type: 'scroll_to', x: 0, y: 500 };
    const result = executeAction(command);

    expect(result.success).toBe(true);
    expect(global.scrollTo).toHaveBeenCalledWith(0, 500);
  });

  test('should return error if ref is not found', () => {
    // Empty map
    generateSnapshot(); // No interactive elements

    const command = { type: 'click_element', ref: 999 };
    const result = executeAction(command);

    expect(result.success).toBe(false);
    expect(result.error).toContain('999 not found');
  });

  test('should return error for unknown action type', () => {
    const command = { type: 'invalid_action' };
    const result = executeAction(command);

    expect(result.success).toBe(false);
    expect(result.error).toContain('Unknown action type');
  });
});
