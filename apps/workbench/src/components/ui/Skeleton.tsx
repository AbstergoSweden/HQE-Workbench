interface SkeletonProps {
  className?: string
  count?: number
}

export function Skeleton({ className = '', count = 1 }: SkeletonProps) {
  if (count > 1) {
    return (
      <div className="space-y-2">
        {Array.from({ length: count }).map((_, i) => (
          <div
            key={i}
            className={`animate-pulse rounded bg-emerald-500/10 ${className}`}
          />
        ))}
      </div>
    )
  }

  return (
    <div className={`animate-pulse rounded bg-emerald-500/10 ${className}`} />
  )
}
