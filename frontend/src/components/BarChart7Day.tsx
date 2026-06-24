import { useMemo, useState } from "react";
import type { DailyBreakdown } from "../types";

interface BarChart7DayProps {
  data: DailyBreakdown[];
}

const VW = 1000, VH = 390;
const PAD = { left: 72, right: 12, top: 8, bottom: 8, gap: 26 };

export default function BarChart7Day({ data }: BarChart7DayProps) {
  const [hovered, setHovered] = useState<{ section: string; dayIdx: number; ico: string } | null>(null);

  const groups = useMemo(() => {
    return data.map((d) => {
      const parts = d.date.split("-");
      return {
        day: parts.length === 3 ? `${parts[1]}-${parts[2]}` : d.date,
        date: d.date,
        input: { audit: d.input_detected, claimed: d.input_claimed },
        cache: { audit: d.cache_detected, claimed: d.cache_claimed },
        output: { audit: d.output_detected, claimed: d.output_claimed },
      };
    });
  }, [data]);

  if (!groups.length) {
    return <div className="flex items-center justify-center h-full text-sm text-gray-500">暂无统计数据</div>;
  }

  // Prefill scale
  const prefillVals = groups.flatMap(g => [g.input.audit, g.input.claimed, g.cache.audit, g.cache.claimed]);
  const prefillMax = Math.max(...prefillVals, 1);
  const prefillYMax = Math.ceil(prefillMax / 1000) * 1000 || 1000;

  // Output scale
  const outputVals = groups.flatMap(g => [g.output.audit, g.output.claimed]);
  const outputMax = Math.max(...outputVals, 1);
  const outputYMax = Math.ceil(outputMax / 1000) * 1000 || 1000;

  const sectionH = (VH - PAD.top - PAD.bottom - PAD.gap) / 2;
  const prefillTop = PAD.top;
  const prefillBottom = prefillTop + sectionH;
  const outputTop = prefillBottom + PAD.gap;
  const outputBottom = outputTop + sectionH;

  const chartW = VW - PAD.left - PAD.right;
  const groupW = chartW / groups.length;
  const barW = Math.max(4, groupW * 0.18);
  const barGap = groupW * 0.04;

  const formatK = (v: number) => v >= 1000 ? `${(v / 1000).toFixed(0)}K` : String(v);
  const fmtY = (v: number) => (v / 1000).toFixed(1).replace(/\.0$/, "") + "K";

  const renderSection = (
    label: string,
    labelColor: string,
    top: number,
    bottom: number,
    yMax: number,
    bars: { ico: string; color: string; audit: number[]; claimed: number[] }[]
  ) => {
    const ch = bottom - top;
    const toY = (v: number) => bottom - (v / yMax) * ch;
    const yTicks = [0, Math.round(yMax / 2), yMax];

    return (
      <g key={label}>
        <text x={VW - PAD.right} y={top + 12}
          fill={labelColor} fontSize="12" fontWeight="700" textAnchor="end"
          style={{ textTransform: "uppercase", letterSpacing: "0.02em" }}>
          {label}
        </text>
        {yTicks.map((v, i) => (
          <g key={i}>
            <line x1={PAD.left} y1={toY(v)} x2={VW - PAD.right} y2={toY(v)}
              stroke="var(--gray-300)" strokeWidth="0.5" />
            <text x={PAD.left - 10} y={toY(v) + 5}
              fill="var(--gray-500)" fontSize="13" fontWeight="500" textAnchor="end"
              dominantBaseline="alphabetic">
              {fmtY(v)}
            </text>
          </g>
        ))}
        {groups.map((g, gi) => {
          const x0 = PAD.left + gi * groupW + groupW / 2;
          return bars.map((bar, bi) => {
            const vals = [bar.audit[gi] ?? 0, bar.claimed[gi] ?? 0];
            const auditVal = vals[0], claimedVal = vals[1];
            const barCenterX = x0 + (bi - (bars.length - 1) / 2) * (barW + barGap);
            const auditH = auditVal > 0 ? (auditVal / yMax) * ch : 0;
            const claimedH = claimedVal > 0 ? (claimedVal / yMax) * ch : 0;
            const isHovered = hovered?.section === label && hovered?.dayIdx === gi && hovered?.ico === bar.ico;

            return (
              <g key={`${gi}-${bar.ico}`}
                onMouseEnter={() => setHovered({ section: label, dayIdx: gi, ico: bar.ico })}
                onMouseLeave={() => setHovered(null)}
                className="cursor-pointer"
              >
                {auditH > 0 && (
                  <rect x={barCenterX - barW / 2} y={toY(auditVal)} width={barW}
                    height={Math.max(auditH, 1)} fill={bar.color}
                    fillOpacity={isHovered ? 1 : 0.9} rx={1.5} />
                )}
                {claimedH > 0 && (
                  <rect x={barCenterX - barW / 2} y={toY(claimedVal)} width={barW}
                    height={Math.max(claimedH, 1)} fill={bar.color}
                    fillOpacity={0.35} rx={1.5} />
                )}
              </g>
            );
          });
        })}
      </g>
    );
  };

  const prefillBars = [
    { ico: "input", color: "var(--ico-input)", audit: groups.map(g => g.input.audit), claimed: groups.map(g => g.input.claimed) },
    { ico: "cache", color: "var(--ico-cache)", audit: groups.map(g => g.cache.audit), claimed: groups.map(g => g.cache.claimed) },
  ];
  const outputBars = [
    { ico: "output", color: "var(--ico-output)", audit: groups.map(g => g.output.audit), claimed: groups.map(g => g.output.claimed) },
  ];

  return (
    <div className="w-full h-full flex flex-col">
      {/* SVG chart — preserveAspectRatio none for bars, but text labels rendered in HTML below */}
      <div className="flex-1 min-h-0 relative">
        <svg viewBox={`0 0 ${VW} ${VH}`} preserveAspectRatio="none"
          style={{ width: "100%", height: "100%", display: "block", position: "absolute", top: 0, left: 0 }}>
          {renderSection("Prefill", "var(--ico-input)", prefillTop, prefillBottom, prefillYMax, prefillBars)}
          {renderSection("Output", "var(--ico-output)", outputTop, outputBottom, outputYMax, outputBars)}

          {/* Tooltip */}
          {hovered && (() => {
            const g = groups[hovered.dayIdx];
            const icoKey = hovered.ico as "input" | "cache" | "output";
            const v = g[icoKey];
            const secBottom = hovered.section === "Prefill" ? prefillBottom : outputBottom;
            const yMax = hovered.section === "Prefill" ? prefillYMax : outputYMax;
            const ch = sectionH;
            const toY = (val: number) => secBottom - (val / yMax) * ch;
            const x0 = PAD.left + hovered.dayIdx * groupW + groupW / 2;
            const barsInSec = hovered.section === "Prefill" ? prefillBars : outputBars;
            const bi = barsInSec.findIndex(b => b.ico === hovered.ico);
            const barCenterX = x0 + (bi - (barsInSec.length - 1) / 2) * (barW + barGap);
            const tW = 200, tH = 56;
            const barTop = toY(Math.max(v.audit, v.claimed));
            const ty = Math.max(2, barTop - tH - 14);
            const tx = Math.min(Math.max(barCenterX - tW / 2, 2), VW - tW - 2);
            return (
              <g>
                <rect x={tx} y={ty} width={tW} height={tH} rx={6} fill="var(--gray-900)" fillOpacity={0.9} />
                <text x={tx + tW / 2} y={ty + 22} fill="white" fontSize="20" textAnchor="middle" fontWeight="700">
                  {icoKey.toUpperCase()}
                </text>
                <text x={tx + tW / 2} y={ty + 46} fill="var(--gray-300)" fontSize="16" textAnchor="middle">
                  审计 {formatK(v.audit)} · 声称 {formatK(v.claimed)}
                </text>
              </g>
            );
          })()}
        </svg>
      </div>

      {/* X-axis labels — HTML, not stretched by preserveAspectRatio */}
      <div className="flex items-center shrink-0"
        style={{ paddingLeft: `${(PAD.left / VW) * 100}%`, paddingRight: `${(PAD.right / VW) * 100}%` }}>
        {groups.map((g, gi) => (
          <div key={gi} className="flex-1 text-center text-xs font-medium text-gray-600">
            {g.day}
          </div>
        ))}
      </div>
    </div>
  );
}
