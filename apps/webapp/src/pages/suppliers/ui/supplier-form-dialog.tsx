import { useForm } from '@tanstack/react-form'
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
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { Textarea } from '#/components/ui/textarea'
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
			<DialogContent className="max-w-xl">
				<DialogHeader>
					<DialogTitle>
						{editingName ? `Modifier ${editingName}` : 'Nouveau fournisseur'}
					</DialogTitle>
					<DialogDescription>
						Les coordonnées reprises dans le DOE et sur les fiches produit.
					</DialogDescription>
				</DialogHeader>

				<form
					className="space-y-4"
					onSubmit={(event) => {
						event.preventDefault()
						event.stopPropagation()
						void form.handleSubmit()
					}}
				>
					<form.Field
						name="name"
						validators={{
							onSubmit: ({ value }) => validateSupplierName(value),
							onChange: ({ value }) => validateSupplierName(value),
						}}
					>
						{(field) => (
							<div className="space-y-1">
								<Label htmlFor="supplier-name">Nom</Label>
								<Input
									id="supplier-name"
									value={field.state.value}
									onChange={(event) => field.handleChange(event.target.value)}
									onBlur={field.handleBlur}
									aria-invalid={field.state.meta.errors.length > 0}
								/>
								{field.state.meta.errors.length > 0 ? (
									<p role="alert" className="text-sm text-destructive">
										{field.state.meta.errors.join(' ')}
									</p>
								) : null}
							</div>
						)}
					</form.Field>

					<div className="grid gap-4 sm:grid-cols-2">
						<form.Field name="registrationNumber">
							{(field) => (
								<div className="space-y-1">
									<Label htmlFor="supplier-registration">SIRET</Label>
									<Input
										id="supplier-registration"
										value={field.state.value}
										onChange={(event) => field.handleChange(event.target.value)}
									/>
								</div>
							)}
						</form.Field>
						<form.Field name="vatNumber">
							{(field) => (
								<div className="space-y-1">
									<Label htmlFor="supplier-vat">N° de TVA</Label>
									<Input
										id="supplier-vat"
										value={field.state.value}
										onChange={(event) => field.handleChange(event.target.value)}
									/>
								</div>
							)}
						</form.Field>
					</div>

					<form.Field name="address">
						{(field) => (
							<div className="space-y-1">
								<Label htmlFor="supplier-address">Adresse</Label>
								<Textarea
									id="supplier-address"
									value={field.state.value}
									onChange={(event) => field.handleChange(event.target.value)}
									placeholder="Une ligne par élément d'adresse"
								/>
							</div>
						)}
					</form.Field>

					<div className="grid gap-4 sm:grid-cols-2">
						<form.Field name="phone">
							{(field) => (
								<div className="space-y-1">
									<Label htmlFor="supplier-phone">Téléphone</Label>
									<Input
										id="supplier-phone"
										type="tel"
										value={field.state.value}
										onChange={(event) => field.handleChange(event.target.value)}
									/>
								</div>
							)}
						</form.Field>
						<form.Field name="email">
							{(field) => (
								<div className="space-y-1">
									<Label htmlFor="supplier-email">Email</Label>
									<Input
										id="supplier-email"
										type="email"
										value={field.state.value}
										onChange={(event) => field.handleChange(event.target.value)}
									/>
								</div>
							)}
						</form.Field>
					</div>

					{error ? <p className="text-sm text-destructive">{error}</p> : null}

					<DialogFooter>
						<Button
							type="button"
							variant="outline"
							onClick={() => onOpenChange(false)}
						>
							Annuler
						</Button>
						<Button type="submit" disabled={isPending}>
							{isPending ? <Loader2 className="animate-spin" /> : null}
							{editingName ? 'Enregistrer' : 'Créer'}
						</Button>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	)
}
