/**
 * E2E test helpers for AI Group Chat
 */

/**
 * Wait for the app to fully load (sidebar visible)
 */
export async function waitForAppReady() {
  const sidebar = await $("h1=AI Group Chat");
  await sidebar.waitForDisplayed({ timeout: 10000 });
}

/**
 * Click the "New Topic" button in the sidebar
 */
export async function clickNewTopic() {
  const btn = await $("button=New Topic");
  await btn.click();
}

/**
 * Click the "Manage Bots" button in the sidebar
 */
export async function clickManageBots() {
  const btn = await $("button=Manage Bots");
  await btn.click();
}

/**
 * Create a bot via the UI
 */
export async function createBot({ name, baseUrl, model, apiKey = "" }) {
  await clickManageBots();
  // Wait for dialog
  const addBtn = await $("button=Add Bot");
  await addBtn.waitForDisplayed();
  await addBtn.click();

  // Fill form
  const nameInput = await $("#bot-name");
  await nameInput.waitForDisplayed();
  await nameInput.setValue(name);

  const urlInput = await $("#bot-url");
  await urlInput.setValue(baseUrl);

  if (apiKey) {
    const keyInput = await $("#bot-key");
    await keyInput.setValue(apiKey);
  }

  const modelInput = await $("#bot-model");
  await modelInput.setValue(model);

  // Submit
  const submitBtn = await $("button=Add Bot");
  await submitBtn.click();

  // Wait a moment for the bot to be saved
  await browser.pause(500);
}

/**
 * Create a topic via the UI
 */
export async function createTopic(title, botNames = []) {
  await clickNewTopic();

  // Fill title
  const titleInput = await $("#topic-title");
  await titleInput.waitForDisplayed();
  await titleInput.setValue(title);

  // Select bots by clicking their checkboxes
  for (const name of botNames) {
    const label = await $(`span=${name}`);
    await label.click();
  }

  // Submit
  const submitBtn = await $("button=Create Topic");
  await submitBtn.click();

  await browser.pause(500);
}

/**
 * Send a message in the current chat
 */
export async function sendMessage(text) {
  const textarea = await $('textarea[placeholder*="Type a message"]');
  await textarea.waitForDisplayed();
  await textarea.setValue(text);

  const sendBtn = await $('button[type="button"]:last-child');
  // Use the Send button (last button in the input row)
  const buttons = await $$("button");
  for (const btn of buttons) {
    const isDisabled = await btn.getAttribute("disabled");
    if (!isDisabled) {
      const svgs = await btn.$$("svg");
      // Find the send button by checking if it's in the input area
      // This is a heuristic - the send button is the last enabled icon button
    }
  }

  // Simpler: press Enter to send
  await textarea.keys("Enter");
  await browser.pause(300);
}

/**
 * Get all message bubbles in the chat
 */
export async function getMessages() {
  const bubbles = await $$(".rounded-xl");
  return bubbles;
}

/**
 * Wait for streaming to complete (no more pulsing cursors)
 */
export async function waitForStreamingComplete(timeout = 30000) {
  await browser.waitUntil(
    async () => {
      const cursors = await $$(".animate-pulse");
      return cursors.length === 0;
    },
    { timeout, timeoutMsg: "Streaming did not complete in time" },
  );
}

/**
 * Get text content of an element
 */
export async function getTextContent(selector) {
  const el = await $(selector);
  return el.getText();
}
