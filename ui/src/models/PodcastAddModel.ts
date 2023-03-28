export type PodcastAddModel = {
    artworkUrl600: string,
    artistName: string,
    collectionName: string,
    trackId: number
}

export type GeneralModel = {
   resultCount: number,
        results: PodcastAddModel[]
}


export type AgnosticPodcastDataModel = {
    imageUrl: string,
    title: string,
    artist: string,
    id: number
}


export type PodIndexModel = {
    feeds: [{
        artwork: string,
        title: string,
        id: number,
        author: string
    }]
}
