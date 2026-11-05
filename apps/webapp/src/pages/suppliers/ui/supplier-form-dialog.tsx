import { useForm } from '@tanstack/react-form'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { cn } from '#/lib/utils'
import { TextArea } from '#/pages/customers/ui/text-area'
import {
	type SupplierFormValues,
	validateSupplierName,
} from '#/pages/suppliers/types'

export interface SupplierFormDialogProps {
	open: boolean
	editingName: string | null
	initialValues: SupplierFormValues
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onSubmit: (values: SupplierFormValues) => void
}

function Section({
	label,
	hint,
	children,
}: {
	label: string
	hint?: string
	children: ReactNode
}) {
	return (
		<section aria-label={label} className="grid gap-3">
			<div>
				<h3 className="font-medium text-sm">{label}</h3>
				{hint && <p className="text-[12px] text-muted-foreground">{hint}</p>}
			</div>
			<div className="grid gap-3 sm:grid-cols-2">{children}</div>
		</section>
	)
}

function Field({
	id,
	label,
	required,
	hint,
	className,
	children,
}: {
	id: string
	label: string
	required?: boolean
	hint?: string
	className?: string
	children: ReactNode
}) {
	return (
		<div className={cn('grid content-start gap-1.5', className)}>
			<Label htmlFor={id}>
				<span>
					{label}
					{required && (
						<span aria-hidden className="ml-0.5 text-state-alert">
							*
						</span>
					)}
				</span>
			</Label>
			{children}
			{hint && <p className="text-[12px] text-muted-foreground">{hint}</p>}
		</div>
	)
}

export function SupplierFormDialog({
	open,
	editingName,
	initialValues,
	isPending,
	error,
	onOpenChange,
	onSubmit,
}: SupplierFormDialogProps) {
	const form = useForm({
		defaultValues: initialValues,
		onSubmit: ({ value }) => {
			onSubmit(value)
		},
	})

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="flex max-h-[calc(100dvh-2rem)] flex-col gap-0 p-0 sm:max-w-2xl">
				<form
					noValidate
					className="flex min-h-0 flex-1 flex-col"
					onSubmit={(event) => {
						event.preventDefault()
						event.stopPropagation()
						void form.handleSubmit()
					}}
				>
					<DialogHeader className="px-6 pt-6 pb-4">
						<DialogTitle>
							{editingName ? `Modifier ${editingName}` : 'Nouveau fournisseur'}
						</DialogTitle>
						<DialogDescription>
							{editingName
								? 'Mets à jour sa fiche. Les coordonnées sont reprises dans le DOE et sur les fiches produit.'
								: 'Renseigne l’essentiel, tu pourras compléter la fiche plus tard.'}
						</DialogDescription>
					</DialogHeader>
					<div className="grid min-h-0 flex-1 gap-6 overflow-y-auto border-t px-6 py-5">
						<Section label="Identité">
							<form.Field
								name="name"
								validators={{
									onSubmit: ({ value }) => validateSupplierName(value),
									onChange: ({ value }) => validateSupplierName(value),
								}}
							>
								{(field) => (
									<Field
										id="supplier-name"
										label="Nom"
										required
										className="sm:col-span-2"
									>
										<Input
											id="supplier-name"
											value={field.state.value}
											onChange={(event) =>
												field.handleChange(event.target.value)
											}
											onBlur={field.handleBlur}
											aria-invalid={field.state.meta.errors.length > 0}
											autoComplete="off"
										/>
										{field.state.meta.errors.length > 0 ? (
											<p role="alert" className="text-[12px] text-state-alert">
												{field.state.meta.errors.join(' ')}
											</p>
										) : null}
									</Field>
								)}
							</form.Field>
							<form.Field name="registrationNumber">
								{(field) => (
									<Field
										id="supplier-registration"
										label="SIRET"
										hint="14 chiffres, espaces acceptés."
									>
										<Input
											id="supplier-registration"
											inputMode="numeric"
											value={field.state.value}
											onChange={(event) =>
												field.handleChange(event.target.value)
											}
											autoComplete="off"
										/>
									</Field>
								)}
							</form.Field>
							<form.Field name="vatNumber">
								{(field) => (
									<Field id="supplier-vat" label="N° de TVA">
										<Input
											id="supplier-vat"
											value={field.state.value}
											onChange={(event) =>
												field.handleChange(event.target.value)
											}
											autoComplete="off"
											placeholder="FR12345678901"
										/>
									</Field>
								)}
							</form.Field>
						</Section>
						<Section label="Contact">
							<form.Field name="phone">
								{(field) => (
									<Field id="supplier-phone" label="Téléphone">
										<Input
											id="supplier-phone"
											type="tel"
											value={field.state.value}
											onChange={(event) =>
												field.handleChange(event.target.value)
											}
											autoComplete="off"
										/>
									</Field>
								)}
							</form.Field>
							<form.Field name="email">
								{(field) => (
									<Field id="supplier-email" label="Email">
										<Input
											id="supplier-email"
											type="email"
											value={field.state.value}
											onChange={(event) =>
												field.handleChange(event.target.value)
											}
											autoComplete="off"
										/>
									</Field>
								)}
							</form.Field>
						</Section>
						<Section label="Adresse">
							<form.Field name="address">
								{(field) => (
									<Field
										id="supplier-address"
										label="Adresse"
										className="sm:col-span-2"
									>
										<TextArea
											id="supplier-address"
											rows={3}
											value={field.state.value}
											onChange={(event) =>
												field.handleChange(event.target.value)
											}
											placeholder="Une ligne par élément d'adresse"
										/>
									</Field>
								)}
							</form.Field>
						</Section>
					</div>
					<div className="grid gap-3 border-t px-6 py-4">
						{error ? (
							<p
								role="alert"
								className="rounded-md bg-state-alert/8 px-3 py-2 text-[13px] text-state-alert"
							>
								{error}
							</p>
						) : null}
						<DialogFooter>
							<Button
								type="button"
								variant="outline"
								onClick={() => onOpenChange(false)}
							>
								Annuler
							</Button>
							<Button type="submit" disabled={isPending}>
								{editingName ? 'Enregistrer' : 'Créer le fournisseur'}
							</Button>
						</DialogFooter>
					</div>
				</form>
			</DialogContent>
		</Dialog>
	)
}
