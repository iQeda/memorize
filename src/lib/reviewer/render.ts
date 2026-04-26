// Minimal port of Anki's reviewer rendering helper.
// Source: ankitects/anki/ts/reviewer/index.ts (replaceScript / setInnerHTML)
// We re-evaluate <script> tags so card templates that ship inline JS still run.
// MathJax / audio / image preload are deferred to later phases.

function replaceScript(oldScript: HTMLScriptElement): Promise<void> {
  return new Promise((resolve) => {
    const newScript = document.createElement("script");
    let mustWaitForNetwork = true;
    if (oldScript.src) {
      newScript.addEventListener("load", () => resolve());
      newScript.addEventListener("error", () => resolve());
    } else {
      mustWaitForNetwork = false;
    }
    for (const attr of oldScript.attributes) {
      newScript.setAttribute(attr.name, attr.value);
    }
    newScript.appendChild(document.createTextNode(oldScript.innerHTML));
    oldScript.replaceWith(newScript);
    if (!mustWaitForNetwork) resolve();
  });
}

export async function setCardHtml(
  el: HTMLElement,
  html: string,
): Promise<void> {
  for (const oldVideo of Array.from(el.getElementsByTagName("video"))) {
    oldVideo.pause();
    while (oldVideo.firstChild) oldVideo.removeChild(oldVideo.firstChild);
    oldVideo.load();
  }
  el.innerHTML = html;
  for (const oldScript of Array.from(el.getElementsByTagName("script"))) {
    await replaceScript(oldScript);
  }
}
