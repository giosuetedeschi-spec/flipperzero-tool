interface Props {
  children: React.ReactNode;
  variant?: "default" | "success" | "warning" | "error";
}

const variants = {
  default: "bg-gray-700 text-gray-300",
  success: "bg-emerald-800 text-emerald-200",
  warning: "bg-amber-800 text-amber-200",
  error: "bg-red-900 text-red-200",
};

export default function Badge({ children, variant = "default" }: Props) {
  return (
    <span className={`px-2 py-0.5 rounded text-xs font-medium ${variants[variant]}`}>
      {children}
    </span>
  );
}
