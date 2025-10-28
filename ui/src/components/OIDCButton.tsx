import { useTranslation } from 'react-i18next'
import { CustomButtonPrimary } from './CustomButtonPrimary'

export const OIDCButton = () => {
	const { t } = useTranslation()

	return (
		<CustomButtonPrimary
			className="text-center w-full"
			onClick={() => {
				window.location.href = '../'
			}}
		>
			{t('oidc-login')}
		</CustomButtonPrimary>
	)
}
