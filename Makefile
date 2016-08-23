.PHONY: doc gh-pages publish

TAG	:= master

gh-pages:
	[[ -d gh-pages ]] && rm -rf gh-pages || echo
	git clone https://github.com/ys-nuem/rust-gurobi.git -b gh-pages gh-pages

doc:
	cargo clean && cargo doc
	mkdir -p gh-pages/$(TAG) || echo
	rm -rf doc/$(TAG)/*
	cp -r target/doc/* gh-pages/$(TAG)/
	rm -f gh-pages/$(TAG)/.lock

publish: gh-pages | doc
	cd gh-pages && git add .
	cd gh-pages && git commit --amend -m "update doc"
	cd gh-pages && git push -f origin gh-pages
