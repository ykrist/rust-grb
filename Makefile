.PHONY: doc gh-pages publish

doc:
	cargo doc

gh-pages:
	[[ ! -d gh-pages ]] && git clone https://github.com/ys-nuem/rust-gurobi.git -b gh-pages gh-pages || echo
	cd gh-pages && git checkout -f gh-pages
	cd gh-pages && git pull --ff

publish: doc | gh-pages
	cp -r target/doc gh-pages
	rm -f gh-pages/.lock
	cd gh-pages && git commit --amend -m "update doc"
	cd gh-pages && git push -f origin gh-pages
