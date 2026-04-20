import assert from "node:assert/strict";

describe("Fricon desktop smoke", () => {
  it("opens a seeded workspace, lists datasets, and renders a chart", async () => {
    const body = await $("body");

    await browser.waitUntil(
      async () => {
        const text = await body.getText();
        return text.includes("Ready") && text.includes("desktop_smoke_signal");
      },
      {
        timeout: 60_000,
        interval: 250,
        timeoutMsg: "expected the desktop shell to load the seeded workspace",
      },
    );

    const datasetRow = await $(
      "//tr[.//*[contains(normalize-space(.), 'desktop_smoke_signal')]]",
    );
    await datasetRow.waitForDisplayed({ timeout: 60_000 });
    await datasetRow.click();

    const chartsTab = await $("aria/Charts");
    await chartsTab.waitForDisplayed({ timeout: 10_000 });

    await browser.waitUntil(
      async () => {
        const noSelection = await $("*=No dataset selected");
        return !(await noSelection.isExisting());
      },
      {
        timeout: 60_000,
        interval: 250,
        timeoutMsg: "expected dataset selection to activate the inspector",
      },
    );

    const canvas = await $("canvas");
    const svg = await $("svg");
    await canvas.waitForDisplayed({ timeout: 60_000 });
    await svg.waitForDisplayed({ timeout: 60_000 });

    const text = await body.getText();
    assert.match(text, /View/);
  });
});
