import {FC} from "react";
import {Skeleton} from "./skeleton";

type LoadingSkeletonProps = {
    loading?: boolean,
    text: string| undefined
}

export const LoadingSkeletonSpan: FC<LoadingSkeletonProps> = ({
                                                                loading,
                                                                text
                                                            }) => {
    return (
        <span className="text-(--fg-secondary-color)">{loading == true ? <Skeleton style={{height: '100%'}}/>: text}</span>
    )
}