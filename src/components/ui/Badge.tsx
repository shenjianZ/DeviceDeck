interface BadgeProps {
  variant:
    | "online"
    | "offline"
    | "unauthorized"
    | "unknown"
    | "system"
    | "adb"
    | "scrcpy"
    | "info"
    | "warn"
    | "error";
  children: React.ReactNode;
  className?: string;
}

const variantClass: Record<BadgeProps["variant"], string> = {
  online: "online",
  offline: "offline",
  unauthorized: "unauthorized",
  unknown: "unknown",
  system: "source-sys",
  adb: "source-adb",
  scrcpy: "source-scrcpy",
  info: "level-info",
  warn: "level-warn",
  error: "level-error",
};

export function Badge({ variant, children, className = "" }: BadgeProps) {
  return (
    <span className={`badge ${variantClass[variant]} ${className}`.trim()}>
      <span className="badge-dot" />
      {children}
    </span>
  );
}
