const prev = "1.0.0";
const next = "1.0.1";
function escapeRegExp(string) {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

const pattern = `jubarte\\s*=\\s*"${escapeRegExp(prev)}"`;
console.log(pattern);
