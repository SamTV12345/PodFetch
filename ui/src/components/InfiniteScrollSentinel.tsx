import {FC, useEffect, useRef} from "react";

type InfiniteScrollSentinelProps = {
    onEnter: () => void
    disabled?: boolean
    rootMargin?: string
    className?: string
}

export const InfiniteScrollSentinel: FC<InfiniteScrollSentinelProps> = ({
    onEnter,
    disabled = false,
    rootMargin = "350px",
    className = ""
}) => {
    const sentinelRef = useRef<HTMLDivElement>(null)
    const hasTriggeredRef = useRef(false)

    useEffect(() => {
        if (disabled || !sentinelRef.current) {
            return
        }

        const observer = new IntersectionObserver(
            (entries) => {
                const [entry] = entries
                if (!entry) {
                    return
                }

                if (!entry.isIntersecting) {
                    hasTriggeredRef.current = false
                    return
                }

                if (hasTriggeredRef.current) {
                    return
                }

                hasTriggeredRef.current = true
                onEnter()
            },
            {rootMargin}
        )

        observer.observe(sentinelRef.current)
        return () => observer.disconnect()
    }, [disabled, onEnter, rootMargin])

    return <div className={className} ref={sentinelRef} aria-hidden="true"/>
}
