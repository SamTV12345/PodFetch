import {Skeleton} from "./skeleton";

export const LoadingPodcastCard = ()=>{
    return <div className="flex flex-col gap-2 items-end">
        <Skeleton className="w-[160px] h-[160px]" />
        <Skeleton className="w-[100px] h-[20px]" />
    </div>
}