interface LogoProps {
  size?: number;
  className?: string;
}

export function Logo({ size = 34, className }: LogoProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 1024 1024"
      className={className}
      aria-hidden="true"
    >
      <defs>
        <linearGradient id="dock-logo-bg" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0" stopColor="#0a1020" />
          <stop offset="1" stopColor="#172744" />
        </linearGradient>
        <linearGradient id="dock-logo-arc" x1="160" y1="220" x2="860" y2="800">
          <stop offset="0" stopColor="#66a6ff" />
          <stop offset="0.55" stopColor="#4f7dff" />
          <stop offset="1" stopColor="#34e2b0" />
        </linearGradient>
        <linearGradient id="dock-logo-bar" x1="0" y1="0" x2="1" y2="0">
          <stop offset="0" stopColor="#6ea8ff" />
          <stop offset="1" stopColor="#3ce0b4" />
        </linearGradient>
      </defs>
      <rect width="1024" height="1024" rx="224" fill="url(#dock-logo-bg)" />
      <circle
        cx="512"
        cy="512"
        r="322"
        fill="none"
        stroke="#263a5c"
        strokeWidth="46"
        strokeLinecap="round"
        strokeDasharray="1410 610"
        transform="rotate(135 512 512)"
      />
      <circle
        cx="512"
        cy="512"
        r="322"
        fill="none"
        stroke="url(#dock-logo-arc)"
        strokeWidth="46"
        strokeLinecap="round"
        strokeDasharray="980 1040"
        transform="rotate(135 512 512)"
      />
      <rect x="342" y="560" width="74" height="126" rx="37" fill="url(#dock-logo-bar)" opacity="0.75" />
      <rect x="475" y="480" width="74" height="206" rx="37" fill="url(#dock-logo-bar)" />
      <rect x="608" y="396" width="74" height="290" rx="37" fill="url(#dock-logo-bar)" opacity="0.9" />
      <circle cx="512" cy="512" r="34" fill="#eaf2ff" />
      <circle cx="512" cy="512" r="15" fill="#3ce0b4" />
    </svg>
  );
}
