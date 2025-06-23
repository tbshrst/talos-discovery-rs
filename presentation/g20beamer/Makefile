N=main
LATEX=xelatex
PANDOC=pandoc

all: main.pdf example.pdf

${N}.pdf: ${N}.tex
	${LATEX} main.tex
	${LATEX} main.tex

%.pdf: %.md
	${PANDOC} -o $@ -s --template g24beamer.pandoc.template -t beamer $< --listings --slide-level 2 --pdf-engine=${LATEX}
