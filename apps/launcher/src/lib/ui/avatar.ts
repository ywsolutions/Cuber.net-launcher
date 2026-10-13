// Временная замена скинам: детерминированная пиксельная «голова» 8×8 по нику.
// Когда появится API скинов, здесь начнут отдаваться настоящие головы.

const CELLS = 8;
const SKINS = [0xf1c9a5, 0xe0ac80, 0xc68642, 0x8d5524, 0xf6d5bd];
const HAIR = [0x3b2a1e, 0x6b4423, 0x262626, 0xc9a15b, 0x8e3b1f, 0x5a4a8a, 0x2f6f5e];
const EYES = [0x3b5bdb, 0x2f9e44, 0x5c3d2e, 0x1c7ed6, 0x7048e8];

const cache = new Map<string, string>();

/** URL картинки 8×8 (показывать с `image-rendering: pixelated`). */
export function avatarUrl(playerName: string | null | undefined): string {
  const key = (playerName?.trim() || "?").toLowerCase();
  let url = cache.get(key);
  if (!url) {
    url = render(key);
    cache.set(key, url);
  }
  return url;
}

function render(name: string): string {
  const rnd = mulberry32(fnv1a(name));
  const pick = <T>(arr: T[]) => arr[Math.floor(rnd() * arr.length)];
  const skin = pick(SKINS);
  const hair = pick(HAIR);
  const eye = pick(EYES);
  const hasBeard = rnd() < 0.25;
  const hasSideburns = rnd() < 0.5;

  const face = new Array<number>(CELLS * CELLS);
  for (let y = 0; y < CELLS; y++)
    for (let x = 0; x < CELLS; x++) {
      let color = skin;
      if (
        y < 2 ||
        (y === 2 && (x === 0 || x === 7 || rnd() < 1 / 3)) ||
        (y === 3 && hasSideburns && (x === 0 || x === 7))
      )
        color = hair;
      face[y * CELLS + x] = color;
    }

  face[4 * CELLS + 1] = 0xffffff;
  face[4 * CELLS + 2] = eye;
  face[4 * CELLS + 5] = eye;
  face[4 * CELLS + 6] = 0xffffff;
  face[5 * CELLS + 3] = shade(skin, 0.82);
  face[5 * CELLS + 4] = shade(skin, 0.82);

  if (hasBeard) {
    for (let y = 6; y < 8; y++) for (let x = 1; x < 7; x++) face[y * CELLS + x] = hair;
    face[6 * CELLS + 3] = shade(hair, 0.7);
    face[6 * CELLS + 4] = shade(hair, 0.7);
  } else {
    for (let x = 2; x < 6; x++) face[6 * CELLS + x] = shade(skin, 0.68);
  }

  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = CELLS;
  const ctx = canvas.getContext("2d")!;
  const img = ctx.createImageData(CELLS, CELLS);
  for (let i = 0; i < face.length; i++) {
    const c = shade(face[i], 0.95 + rnd() * 0.1);
    img.data[i * 4] = (c >> 16) & 0xff;
    img.data[i * 4 + 1] = (c >> 8) & 0xff;
    img.data[i * 4 + 2] = c & 0xff;
    img.data[i * 4 + 3] = 255;
  }
  ctx.putImageData(img, 0, 0);
  return canvas.toDataURL();
}

function shade(rgb: number, f: number): number {
  const ch = (v: number) => Math.max(0, Math.min(255, Math.round(v * f)));
  return (ch((rgb >> 16) & 0xff) << 16) | (ch((rgb >> 8) & 0xff) << 8) | ch(rgb & 0xff);
}

function fnv1a(text: string): number {
  let h = 0x811c9dc5;
  for (let i = 0; i < text.length; i++) {
    h ^= text.charCodeAt(i);
    h = Math.imul(h, 0x01000193);
  }
  return h >>> 0;
}

function mulberry32(seed: number) {
  return () => {
    seed = (seed + 0x6d2b79f5) | 0;
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
