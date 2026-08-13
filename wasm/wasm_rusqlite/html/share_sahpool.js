// share_sahpool.js – main-thread SAHPool coordination via localStorage
const WANT_KEY = 'sahpool_want';
const me = crypto.randomUUID ? crypto.randomUUID() : Math.random().toString(36).slice(2);

export function GiveMeSahpool() {
  localStorage.setItem(WANT_KEY, me);
}

export function DoesSomebodyElseWantSahpool() {
  const want = localStorage.getItem(WANT_KEY);
  return want !== null && want !== me;
}

export function MyId() {
  return me;
}
