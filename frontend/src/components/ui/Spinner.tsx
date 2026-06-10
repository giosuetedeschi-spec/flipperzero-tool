export default function Spinner({ className = "" }: { className?: string }) {
  return <span className={`animate-spin inline-block ${className}`}>⏳</span>;
}
