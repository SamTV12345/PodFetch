import useCommon from '../store/CommonSlice'
import { CustomButtonPrimary } from './CustomButtonPrimary'
import { CustomButtonSecondary } from './CustomButtonSecondary'
import { Modal } from './Modal'

export type ConfirmModalProps = {
	headerText: string
	onAccept: () => void
	onReject: () => void
	acceptText: string
	rejectText: string
	bodyText: string
}

export const ConfirmModal = () => {
	const confirmModalData = useCommon((state) => state.confirmModalData)

	return (
		<Modal
			acceptText={confirmModalData?.acceptText}
			headerText={confirmModalData?.headerText}
			onAccept={() => {}}
			cancelText={confirmModalData?.rejectText}
			onCancel={() => {}}
			onDelete={() => {}}
		>
			<div className="mb-4 text-(--fg-color)">{confirmModalData?.bodyText}</div>
			<div className="text-right">
				<CustomButtonSecondary
					className="border-transparent shadow-none hover:shadow-none text-base text-(--fg-color) hover:text-(--fg-color-hover)"
					onClick={confirmModalData?.onReject}
				>
					{confirmModalData?.rejectText}
				</CustomButtonSecondary>
				<CustomButtonPrimary
					className="bg-(--danger-fg-color) hover:bg-(--danger-fg-color-hover) hover:shadow-(--danger-fg-color-hover) text-(--fg-color)"
					onClick={confirmModalData?.onAccept}
				>
					{confirmModalData?.acceptText}
				</CustomButtonPrimary>
			</div>
		</Modal>
	)
}
