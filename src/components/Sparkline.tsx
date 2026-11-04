import { useId } from "react";

interface SparklineProps {
  values: number[];
  labels?: string[];
  formatValue?: (value: number) => string;
}

export function Sparkline({ values, labels, formatValue }: SparklineProps) {
  const gradientId = useId().replace(/:/g, "");
  const width = 100;
  const height = 38;
  const padding = 3;
  const max = Math.max(...values, 0.01);
  const min = Math.min(...values, 0);
  const span = Math.max(max - min, 0.01);

  const points = values.map((value, index) => {
    const x =
      values.length <= 1
        ? width / 2
        : padding + (index / (values.length - 1)) * (width - padding * 2);
    const y = height - padding - ((value - min) / span) * (height - padding * 2);
    return { x, y, value, label: labels?.[index] ?? "" };
  });

  const linePath = points
    .map((point, index) => `${index === 0 ? "M" : "L"}${point.x.toFixed(2)},${point.y.toFixed(2)}`)
    .join(" ");
  const areaPath = points.length
    ? `${linePath} L${points[points.length - 1].x.toFixed(2)},${height} L${points[0].x.toFixed(2)},${height} Z`
    : "";

  return (
    <svg
      className="sparkline"
      viewBox={`0 0 ${width} ${height}`}
      preserveAspectRatio="none"
      role="img"
      aria-label="Daily cost trend"
    >
      <defs>
        <linearGradient id={`spark-${gradientId}`} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor="var(--accent)" stopOpacity="0.35" />
          <stop offset="1" stopColor="var(--accent)" stopOpacity="0" />
        </linearGradient>
      </defs>
      {areaPath ? <path d={areaPath} fill={`url(#spark-${gradientId})`} /> : null}
      {linePath ? (
        <path
          d={linePath}
          fill="none"
          stroke="var(--accent)"
          strokeWidth="1.6"
          strokeLinecap="round"
          strokeLinejoin="round"
          vectorEffect="non-scaling-stroke"
        />
      ) : null}
      {points.map((point, index) => (
        <circle
          key={`${point.x}-${index}`}
          cx={point.x}
          cy={point.y}
          r={index === points.length - 1 ? 2.2 : 1.2}
          fill={index === points.length - 1 ? "var(--accent-strong)" : "var(--accent)"}
          opacity={index === points.length - 1 ? 1 : 0.55}
        >
          <title>
            {point.label} {formatValue ? formatValue(point.value) : point.value.toFixed(2)}
          </title>
        </circle>
      ))}
    </svg>
  );
}
