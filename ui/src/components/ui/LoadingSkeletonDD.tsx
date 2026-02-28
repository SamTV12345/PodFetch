import {FC} from "react";
import {Skeleton} from "./skeleton";

type LoadingSkeletonProps = {
    loading?: boolean,
    text: string| undefined|number
}

export const LoadingSkeletonDD: FC<LoadingSkeletonProps> = ({
    loading,
    text
                                                          }) => {
    return (
        <dd className="ui-text-muted">{loading == true ? <Skeleton style={{height: '100%'}}/>: text}</dd>
    )
}