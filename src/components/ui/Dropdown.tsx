import { useEffect, useRef, useState, useCallback } from "react";
import { createPortal } from "react-dom";
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
  const [highlightIndex, setHighlightIndex] = useState(-1);
  const [panelStyle, setPanelStyle] = useState<React.CSSProperties>({});
  const ref = useRef<HTMLDivElement>(null);
  const panelRef = useRef<HTMLDivElement>(null);
  const optionRefs = useRef<(HTMLDivElement | null)[]>([]);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node) &&
          panelRef.current && !panelRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  useEffect(() => {
    if (open) {
      const idx = options.findIndex((o) => o.value === value);
      setHighlightIndex(idx >= 0 ? idx : 0);
    }
  }, [open, options, value]);

  useEffect(() => {
    if (open && highlightIndex >= 0 && optionRefs.current[highlightIndex]) {
      optionRefs.current[highlightIndex]?.scrollIntoView({
        block: "nearest",
        behavior: "smooth",
      });
    }
  }, [highlightIndex, open]);

  useEffect(() => {
    if (!open || !ref.current) return;
    const rect = ref.current.getBoundingClientRect();
    const style: React.CSSProperties = {
      position: "fixed",
      left: rect.left,
      top: rect.bottom + 4,
      minWidth: rect.width,
    };
    const spaceBelow = window.innerHeight - rect.bottom - 4;
    if (spaceBelow < 200 && rect.top > spaceBelow) {
      style.top = undefined;
      style.bottom = window.innerHeight - rect.top + 4;
    }
    setPanelStyle(style);
  }, [open]);

  const selected = options.find((o) => o.value === value);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (!open) {
        if (e.key === "ArrowDown" || e.key === "ArrowUp" || e.key === "Enter") {
          e.preventDefault();
          setOpen(true);
        }
        return;
      }

      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setHighlightIndex((prev) =>
            prev < options.length - 1 ? prev + 1 : 0
          );
          break;
        case "ArrowUp":
          e.preventDefault();
          setHighlightIndex((prev) =>
            prev > 0 ? prev - 1 : options.length - 1
          );
          break;
        case "Enter":
        case " ":
          e.preventDefault();
          if (highlightIndex >= 0 && highlightIndex < options.length) {
            onChange(options[highlightIndex].value);
            setOpen(false);
          }
          break;
        case "Escape":
          e.preventDefault();
          setOpen(false);
          break;
        case "Home":
          e.preventDefault();
          setHighlightIndex(0);
          break;
        case "End":
          e.preventDefault();
          setHighlightIndex(options.length - 1);
          break;
      }
    },
    [open, highlightIndex, options, onChange]
  );

  const panel = open && (
    <div ref={panelRef} className="dd-panel" role="listbox" style={panelStyle}>
      {options.map((o, i) => (
        <div
          key={o.value}
          ref={(el) => { optionRefs.current[i] = el; }}
          className={`dd-opt${o.value === value ? " act" : ""}${
            i === highlightIndex ? " highlight" : ""
          }`}
          role="option"
          aria-selected={o.value === value}
          onClick={() => {
            onChange(o.value);
            setOpen(false);
          }}
          onMouseEnter={() => setHighlightIndex(i)}
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
  );

  return (
    <div ref={ref} className={`dd ${className}`.trim()} onKeyDown={handleKeyDown}>
      <button
        className="dd-trigger"
        onClick={() => setOpen(!open)}
        type="button"
        aria-expanded={open}
        aria-haspopup="listbox"
      >
        <span className="dd-label">{selected?.label ?? placeholder}</span>
        <span className={`dd-chevron${open ? " open" : ""}`}>
          <ChevronDown size={14} />
        </span>
      </button>
      {panel && createPortal(panel, document.body)}
    </div>
  );
}
