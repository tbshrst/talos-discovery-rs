LATEX=xelatex
PANDOC=pandoc

all: presentation.pdf

%.pdf: %.md
	TTFONTS=$$TTFONTS:./g20beamer \
	TEXINPUTS=$$TEXINPUTS:./g20beamer \
	${PANDOC} -o $@ -s --template ./g20beamer/g24beamer.pandoc.template -t beamer $< --listings --slide-level 2 --pdf-engine=${LATEX}
