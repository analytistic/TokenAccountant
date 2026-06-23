import { type ButtonHTMLAttributes } from "react";

type ButtonVariant = "primary" | "secondary" | "danger" | "ghost";
type ButtonSize = "sm" | "md" | "lg";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
}

const variantClasses: Record<ButtonVariant, string> = {
  primary:
    "bg-brand text-white hover:bg-brand-hover shadow-sm hover:shadow-md active:scale-[0.98] transition-all duration-fast ease-out",
  secondary:
    "bg-gray-100 text-gray-700 hover:bg-gray-200 active:bg-gray-300 transition-colors duration-fast",
  danger:
    "bg-danger-subtle text-danger border border-danger-border hover:bg-danger hover:text-white transition-colors duration-fast",
  ghost:
    "bg-transparent text-gray-600 hover:bg-gray-100 active:bg-gray-200 transition-colors duration-fast",
};

const sizeClasses: Record<ButtonSize, string> = {
  sm: "px-3 py-1.5 text-xs rounded-md",
  md: "px-4 py-2 text-sm rounded-lg",
  lg: "px-6 py-2.5 text-base rounded-lg",
};

export default function Button({
  variant = "primary",
  size = "md",
  className = "",
  disabled,
  children,
  ...rest
}: ButtonProps) {
  return (
    <button
      className={`inline-flex items-center justify-center font-medium tracking-tight
        ${variantClasses[variant]}
        ${sizeClasses[size]}
        ${disabled ? "opacity-50 cursor-not-allowed" : "cursor-pointer"}
        ${className}`}
      disabled={disabled}
      {...rest}
    >
      {children}
    </button>
  );
}
