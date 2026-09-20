.PHONY: all gen

all: gen

gen:
	cd src && cd core && python3 _pitch.py
