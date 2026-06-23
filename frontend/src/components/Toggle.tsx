interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
}

export default function Toggle({ checked, onChange, disabled }: ToggleProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={`relative inline-flex items-center rounded-full transition-all duration-normal ease-spring
        ${disabled ? "opacity-50 cursor-not-allowed" : "cursor-pointer"}
        ${checked ? "bg-brand" : "bg-gray-300"}
        w-10 h-6`}
    >
      <span
        className={`inline-block w-4 h-4 bg-white rounded-full shadow-sm transition-transform duration-normal ease-spring
          ${checked ? "translate-x-[22px]" : "translate-x-[3px]"}`}
      />
    </button>
  );
}
