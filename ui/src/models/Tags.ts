interface Tags {
    id: string,
    name: string,
    color: string,
    username: string
}

export type TagCreate = {
    name: string,
    color: string
}