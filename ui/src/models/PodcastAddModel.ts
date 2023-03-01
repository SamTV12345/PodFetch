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
