all: presentation.pdf

presentation.pdf: slides.md
	pandoc -s --pdf-engine xelatex -t beamer $< -o $@
