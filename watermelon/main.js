import init, { GameState } from "./pkg/watermelon.js";

const canvas = document.querySelector("#game");
const ctx = canvas.getContext("2d");

const colors = [
  "#f94144",
  "#f8961e",
  "#f9c74f",
  "#90be6d",
  "#43aa8b",
  "#577590",
  "#9b5de5",
  "#00bbf9",
  "#00f5d4",
  "#2d6a4f",
];

let game;

function canvasPointFromEvent(event) {
  const rect = canvas.getBoundingClientRect();
  const clientX = event.clientX ?? event.touches?.[0]?.clientX;
  const clientY = event.clientY ?? event.touches?.[0]?.clientY;

  return {
    x: ((clientX - rect.left) / rect.width) * canvas.width,
    y: ((clientY - rect.top) / rect.height) * canvas.height,
  };
}

function addFruitFromEvent(event) {
  event.preventDefault();
  const point = canvasPointFromEvent(event);

  // どのサイズを落とすかは WASM 側の重み付き抽選に任せる。
  game.add_random_fruit(point.x, 36);
}

function drawWalls() {
  ctx.save();
  ctx.strokeStyle = "#293241";
  ctx.lineWidth = 8;
  ctx.beginPath();
  ctx.moveTo(0, 0);
  ctx.lineTo(0, canvas.height);
  ctx.lineTo(canvas.width, canvas.height);
  ctx.lineTo(canvas.width, 0);
  ctx.stroke();
  ctx.restore();
}

function drawFruit(fruit) {
  const { x, y, radius, level } = fruit;

  ctx.save();
  ctx.beginPath();
  ctx.arc(x, y, radius, 0, Math.PI * 2);
  ctx.fillStyle = colors[level] ?? "#adb5bd";
  ctx.fill();
  ctx.lineWidth = Math.max(2, radius * 0.08);
  ctx.strokeStyle = "rgba(23, 32, 42, 0.28)";
  ctx.stroke();

  ctx.fillStyle = "rgba(255, 255, 255, 0.88)";
  ctx.beginPath();
  ctx.arc(x - radius * 0.28, y - radius * 0.28, radius * 0.18, 0, Math.PI * 2);
  ctx.fill();

  ctx.fillStyle = "#17202a";
  ctx.font = `${Math.max(10, radius * 0.32)}px system-ui, sans-serif`;
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(String(level + 1), x, y);
  ctx.restore();
}

function render() {
  game.step();

  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = "#fffdf7";
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  drawWalls();

  for (const fruit of game.get_fruits()) {
    drawFruit(fruit);
  }

  requestAnimationFrame(render);
}

async function main() {
  await init();
  game = new GameState();

  canvas.addEventListener("pointerdown", addFruitFromEvent);
  render();
}

main().catch((error) => {
  console.error(error);
  ctx.fillStyle = "#b00020";
  ctx.font = "16px system-ui, sans-serif";
  ctx.fillText("WASM の読み込みに失敗しました", 24, 32);
});
