/**
 * SSE Stream Parser for Chrome Extension
 * Handles raw text tokens from backend SSE stream.
 *
 * Usage:
 * const response = await fetch('/agent/run', { method: 'POST', ... });
 * for await (const event of readSSEStream(response)) {
 *   if (event.type === 'data') {
 *     console.log(event.value);
 *   }
 * }
 */
async function* readSSEStream(response) {
  const reader = response.body.getReader();
  const decoder = new TextDecoder('utf-8');
  let buffer = '';

  try {
    while (true) {
      const { done, value } = await reader.read();

      if (done) {
        buffer += decoder.decode(); // Final flush
        break;
      }

      // Decode the chunk and add it to our buffer
      // TextDecoder with {stream: true} correctly handles multibyte character splitting
      buffer += decoder.decode(value, { stream: true });

      // SSE events are separated by double newlines (\n\n)
      // Split by double newline to get complete events
      const events = buffer.split(/\r?\n\r?\n/);

      // Keep the last (potentially incomplete) event in the buffer
      buffer = events.pop() || '';

      for (const eventBlock of events) {
        if (!eventBlock.trim()) continue;

        // Parse the event block
        let currentEvent = 'data';
        let dataLines = [];

        const lines = eventBlock.split(/\r?\n/);
        for (const line of lines) {
          if (line.startsWith('event:')) {
            currentEvent = line.slice(6).trim();
          } else if (line.startsWith('data:')) {
            let dataValue = line.slice(5);
            // Standard SSE: data: <optional space><content>
            if (dataValue.startsWith(' ')) {
              dataValue = dataValue.slice(1);
            }
            dataLines.push(dataValue);
          }
        }

        // SSE spec: multiple data: lines should be joined with \n
        const fullData = dataLines.join('\n');

        if (fullData.trim() === '[DONE]') {
          yield { type: 'done', value: '' };
          return;
        }

        if (currentEvent === 'error') {
          yield { type: 'error', value: fullData };
        } else if (currentEvent === 'usage') {
          yield { type: 'usage', value: fullData };
        } else if (currentEvent === 'tool') {
          yield { type: 'tool', value: fullData };
        } else {
          yield { type: 'data', value: fullData };
        }
      }
    }

    // Flush remaining buffer if it looks like a complete event
    if (buffer.trim()) {
      let currentEvent = 'data';
      let dataLines = [];

      const lines = buffer.split(/\r?\n/);
      for (const line of lines) {
        if (line.startsWith('event:')) {
          currentEvent = line.slice(6).trim();
        } else if (line.startsWith('data:')) {
          let dataValue = line.slice(5);
          if (dataValue.startsWith(' ')) {
            dataValue = dataValue.slice(1);
          }
          dataLines.push(dataValue);
        }
      }

      if (dataLines.length > 0) {
        const fullData = dataLines.join('\n');

        if (fullData.trim() === '[DONE]') {
          yield { type: 'done', value: '' };
        } else if (currentEvent === 'error') {
          yield { type: 'error', value: fullData };
        } else if (currentEvent === 'usage') {
          yield { type: 'usage', value: fullData };
        } else if (currentEvent === 'tool') {
          yield { type: 'tool', value: fullData };
        } else {
          yield { type: 'data', value: fullData };
        }
      }
    }
  } catch (error) {
    yield { type: 'error', value: error.message || 'Stream connection lost' };
  } finally {
    reader.releaseLock();
  }
}

// Expose to window global for non-module extension environment
window.readSSEStream = readSSEStream;
