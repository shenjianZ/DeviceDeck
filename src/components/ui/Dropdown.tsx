import { useEffect, useRef, useState } from "react";
import { Check, ChevronDown } from "lucide-react";

export interface DropdownOption {
  value: string;
  label: string;
}

interface DropdownProps {
  value: string;
  onChange: (value: string) => void;
  options: DropdownOption[];
  placeholder?: string;
  className?: string;
}

export function Dropdown({
  value,
  onChange,
  options,
  placeholder = "—",
  className = "",
}: DropdownProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  const selected = options.find((o) => o.value === value);

  return (
    <div ref={ref} className={`dd ${className}`.trim()}>
      <button className="dd-trigger" onClick={() => setOpen(!open)} type="button">
        <span className="dd-label">{selected?.label ?? placeholder}</span>
        <span className={`dd-chevron${open ? " open" : ""}`}>
          <ChevronDown size={14} />
        </span>
      </button>
      {open && (
        <div className="dd-panel">
          {options.map((o) => (
            <div
              key={o.value}
              className={`dd-opt${o.value === value ? " act" : ""}`}
              onClick={() => {
                onChange(o.value);
                setOpen(false);
              }}
            >
              {o.label}
              {o.value === value && (
                <span className="dd-check">
                  <Check size={13} />
                </span>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
