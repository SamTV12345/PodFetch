export type FileItem = {
	name: string
	content: string
	json: MyFile
	exists: boolean
}

export interface MyFile {
	lastOpened: string
	content: string
	name: string
	id: string
	repo?: string
}

export const readFile = (file: File): Promise<FileItem> => {
	return new Promise((res, _rej) => {
		const fileItem: FileItem = {
			name: file.name,
			content: '',
			json: { content: '', id: '', name: '', lastOpened: '', repo: '' },
			exists: false,
		}

		const fr = new FileReader()

		fr.onload = async () => {
			const result = fr.result
			if (typeof result === 'string') {
				fileItem.content = result
				res(fileItem)
			}
		}
		fr.readAsText(file)
	})
}
