import { useLayoutEffect, useRef } from 'react'

function useOnMount(callback: any) {
	const hasRunRef = useRef(false)

	useLayoutEffect(() => {
		if (!hasRunRef.current) {
			callback()
			hasRunRef.current = true
		}
	}, [callback])
}

export default useOnMount
