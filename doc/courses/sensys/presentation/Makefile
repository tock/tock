all: presentation.pdf

presentation.pdf: slides.md
	pandoc -s --latex-engine xelatex -t beamer $< -o $@
