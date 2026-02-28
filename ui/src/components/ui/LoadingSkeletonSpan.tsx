import {FC} from "react";
import {Skeleton} from "./skeleton";

type LoadingSkeletonProps = {
    loading?: boolean,
    text?: string| undefined,
    height?: string,
    width?: string
}

export const LoadingSkeletonSpan: FC<LoadingSkeletonProps> = ({
                                                                loading,
                                                                text,
    height, width
                                                            }) => {
    return (
        <span className="ui-text-muted">{loading == true ? <Skeleton style={{height: height??'100%', width: width?? '100%'}}/>: text}</span>
    )
}