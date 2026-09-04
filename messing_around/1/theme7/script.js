const SIZE = 4000;
const GAP = 30;
const BOX1_PUSH = 0;
const BOX2_PUSH = 0;

const basePx = (SIZE + GAP) / (2 * Math.sqrt(2));
const offsetPx1 = basePx + BOX1_PUSH;
const offsetPx2 = basePx + BOX2_PUSH;

document.documentElement.style.setProperty('--size', `${SIZE}px`);

const box1 = document.querySelector('.box1');
const box2 = document.querySelector('.box2');
const buttons = document.querySelectorAll('.nav-btn');

let currentCX = 50;
let currentCY = 50;

function updatePositions(cx, cy, delayBox1 = 0, delayBox2 = 0) {
    box1.style.left = `calc(${cx}% - ${offsetPx1}px)`;
    box1.style.top = `calc(${cy}% + ${offsetPx1}px)`;
    box1.style.transitionDelay = `${delayBox1}ms`;

    box2.style.left = `calc(${cx}% + ${offsetPx2}px)`;
    box2.style.top = `calc(${cy}% - ${offsetPx2}px)`;
    box2.style.transitionDelay = `${delayBox2}ms`;

    currentCX = cx;
    currentCY = cy;
}

updatePositions(50, 50, 0, 0);

buttons.forEach(btn => {
    btn.addEventListener('click', () => {
        const targetCX = parseFloat(btn.dataset.cx);
        const targetCY = parseFloat(btn.dataset.cy);

        const vw = window.innerWidth;
        const vh = window.innerHeight;

        // Current pixel positions of boxes (center point)
        const box1X = (currentCX / 100) * vw - offsetPx1;
        const box1Y = (currentCY / 100) * vh + offsetPx1;
        const box2X = (currentCX / 100) * vw + offsetPx2;
        const box2Y = (currentCY / 100) * vh - offsetPx2;

        // Target pixel position
        const targetX = (targetCX / 100) * vw;
        const targetY = (targetCY / 100) * vh;

        const distBox1 = Math.hypot(box1X - targetX, box1Y - targetY);
        const distBox2 = Math.hypot(box2X - targetX, box2Y - targetY);

        const delayBox1 = distBox1 <= distBox2 ? 0 : 200;
        const delayBox2 = distBox2 < distBox1 ? 0 : 200;

        updatePositions(targetCX, targetCY, delayBox1, delayBox2);
    });
});

window.addEventListener('resize', () => {
    updatePositions(currentCX, currentCY, 0, 0);
});
