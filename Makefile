PHONY += all
all:

PHONY += docker
docker:
	docker build -t rustfully-syndicated .

PHONY += mrproper
mrproper:
	docker image rm rustfully-syndicated:latest

.PHONY: $(PHONY)
