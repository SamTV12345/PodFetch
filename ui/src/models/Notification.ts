export interface Notification {
    id: number,
    message: string,
    typeOfMessage: "success"|"error"|"info"|"warning",
    createdAt: string,
    status: string
}
