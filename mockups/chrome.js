const PAGES = ["workspace", "recording", "revise", "review", "handoff", "history", "hidden"];
const d = document.body.dataset;
const desk = document.querySelector(".desktop");
desk.insertAdjacentHTML("afterbegin", `
  <div class="waybar">
    <div class="ws"><span>󰣇</span><b>1</b><span>2</span><span>3</span><span>4</span><span>5</span></div>
    <div>${d.time}</div>
    <div class="right"><span>󰒄</span><span>󱙺</span>${d.widget}<span>󰂯</span><span>󰖩</span><span>󰋋</span><span>󰐥</span></div>
  </div>
  ${d.noTiles === undefined ? `<div class="tiles">
<div class="tile"><span class="g">herdr · voice-brief · w3:p1 claude</span>
● Running the spike validator tests
  Ran 1 shell command

~/Projects/mine/voice-brief  main  Opus 5.5</div>
<div class="tile">herdr · voice-brief · w3:p2 shell

$ br ready

~  Opus 5.5</div>
  </div>` : ""}
  ${d.noScrim === undefined ? `<div class="scrim"></div>` : ""}`);
desk.insertAdjacentHTML("beforeend", `<nav class="mocks">${PAGES.map((p, i) =>
  `<a class="${d.page === p ? "on" : ""}" href="0${i + 1}-${p}.html">${i + 1} ${p}</a>`).join("")}</nav>`);
