import {FC} from "react";
import {Podcast} from "../store/CommonSlice";
import {Link} from "react-router-dom";

type CardProps = {
    podcast: Podcast
}


export const Card:FC<CardProps> = ({podcast})=>{
    return <div className="max-w-sm bg-white border border-gray-200 rounded-lg shadow dark:bg-gray-800 dark:border-gray-700">
        <Link to={"/podcasts/"+podcast.id}>
            <img className="rounded-t-lg" src={podcast.image_url} alt=""/>
        </Link>
        <div className="p-5">
                <h5 className="mb-2 text-2xl font-bold tracking-tight text-gray-900 dark:text-white">{podcast.name}</h5>
        </div>
    </div>
}
