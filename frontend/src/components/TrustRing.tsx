interface TrustRingProps {
  value: number; // 0-100
  size?: "sm" | "md";
}

const sizeMap = { sm: 40, md: 56 };
const strokeW = { sm: 4, md: 5 };

function color(value: number) {
  if (value >= 90) return "var(--success)";
  if (value >= 70) return "var(--warning)";
  return "var(--danger)";
}

export default function TrustRing({ value, size = "md" }: TrustRingProps) {
  const clamped = Math.max(0, Math.min(100, value));
  const dim = sizeMap[size];
  const sw = strokeW[size];
  const r = (dim - sw) / 2;
  const circumference = 2 * Math.PI * r;
  const dash = (clamped / 100) * circumference;

  return (
    <div className="group relative inline-flex items-center justify-center">
      <svg
        width={dim}
        height={dim}
        viewBox={`0 0 ${dim} ${dim}`}
        className="-rotate-90 transition-all duration-normal ease-out
          group-hover:drop-shadow-[0_0_6px_var(--brand-glow)]"
      >
        {/* Background track */}
        <circle
          cx={dim / 2}
          cy={dim / 2}
          r={r}
          fill="none"
          stroke="var(--gray-200)"
          strokeWidth={sw}
        />
        {/* Progress arc */}
        <circle
          cx={dim / 2}
          cy={dim / 2}
          r={r}
          fill="none"
          stroke={color(clamped)}
          strokeWidth={sw}
          strokeLinecap="round"
          strokeDasharray={circumference}
          strokeDashoffset={circumference - dash}
          className="transition-all duration-slow ease-out"
        />
      </svg>
      <span
        className={`absolute font-semibold text-gray-900 transition-transform duration-normal ease-spring
          group-hover:scale-110
          ${size === "sm" ? "text-[10px]" : "text-xs"}`}
      >
        {Math.round(clamped)}
      </span>
    </div>
  );
}
