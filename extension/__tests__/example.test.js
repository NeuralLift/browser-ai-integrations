describe('Example Test', () => {
  test('should pass', () => {
    expect(true).toBe(true);
  });

  test('jsdom works', () => {
    document.body.innerHTML = '<div id="test">Hello</div>';
    const element = document.getElementById('test');
    expect(element.textContent).toBe('Hello');
  });
});
