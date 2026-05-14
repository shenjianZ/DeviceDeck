interface ToggleProps {
  on: boolean;
  onChange: (on: boolean) => void;
}

export function Toggle({ on, onChange }: ToggleProps) {
  return (
    <button
      className={`toggle${on ? " on" : ""}`}
      onClick={() => onChange(!on)}
      type="button"
    />
  );
}
