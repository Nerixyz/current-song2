default: serve

serve-venv:
	@python3 -m venv venv && source venv/bin/activate \
	&& pip install -r requirements.txt \
	&& mkdocs serve

serve:
	@pip install -r requirements.txt \
	&& mkdocs serve
