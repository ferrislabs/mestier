import { Loader2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import type { MemberFormValues } from '#/pages/hr/types'
import { MemberFormFields } from '#/pages/hr/ui/rh/members/member-form-fields'

export interface MemberCostDialogProps {
	idPrefix: string
	open: boolean
	title: string
	description: string
	submitLabel: string
	values: MemberFormValues
	isPending: boolean
	onChange: (patch: Partial<MemberFormValues>) => void
	onSubmit: () => void
	onOpenChange: (open: boolean) => void
}

export function MemberCostDialog({
	idPrefix,
	open,
	title,
	description,
	submitLabel,
	values,
	isPending,
	onChange,
	onSubmit,
	onOpenChange,
}: MemberCostDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="flex max-h-[85vh] flex-col gap-0 overflow-hidden">
				<DialogHeader className="border-b pb-4">
					<DialogTitle>{title}</DialogTitle>
					<DialogDescription>{description}</DialogDescription>
				</DialogHeader>
				<div className="flex-1 overflow-y-auto py-4">
					<MemberFormFields
						idPrefix={idPrefix}
						values={values}
						onChange={onChange}
					/>
				</div>
				<DialogFooter className="border-t pt-4">
					<Button
						type="button"
						variant="ghost"
						onClick={() => onOpenChange(false)}
					>
						Annuler
					</Button>
					<Button onClick={onSubmit} disabled={isPending}>
						{isPending ? <Loader2 className="animate-spin" /> : null}
						{submitLabel}
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
