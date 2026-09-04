const SIZE = 4000;       // box side length (px)
const GAP = 40;         // base symmetric gap (px)
const BOX1_PUSH = 0;    // extra px to move box1 outward (increases gap)
const BOX2_PUSH = 0;    // extra px to move box2 outward (increases gap)

const base = (SIZE + GAP) / (2 * Math.sqrt(2));
const box1Offset = base + BOX1_PUSH;
const box2Offset = base + BOX2_PUSH;

const root = document.documentElement.style;
root.setProperty('--size', `${SIZE}px`);
root.setProperty('--box1-offset', `${box1Offset}px`);
root.setProperty('--box2-offset', `${box2Offset}px`);
