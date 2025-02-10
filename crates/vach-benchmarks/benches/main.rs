use std::io;

use criterion::{Criterion, black_box, criterion_group, criterion_main, Throughput};
use vach::prelude::*;
use vach::crypto_utils::{gen_keypair, read_verifying_key};

// Remove io overhead by Sinking data into the void
struct Sink(u64);

impl Sink {
	fn new() -> Sink {
		black_box(Sink(0))
	}
}

impl io::Seek for Sink {
	fn seek(&mut self, seek: io::SeekFrom) -> io::Result<u64> {
		match seek {
			io::SeekFrom::Start(s) => self.0 = s,
			io::SeekFrom::Current(s) => self.0 = (self.0 as i64 + s) as u64,
			io::SeekFrom::End(s) => self.0 = (self.0 as i64 + s) as u64,
		}

		Ok(self.0)
	}
}

impl io::Write for Sink {
	fn write(&mut self, sequence: &[u8]) -> io::Result<usize> {
		Ok(sequence.len())
	}

	fn flush(&mut self) -> io::Result<()> {
		Ok(())
	}
}

pub fn criterion_benchmark(c: &mut Criterion) {
	let keypair_bytes = &gen_keypair().to_keypair_bytes() as &[u8];

	let mut b_config = BuilderConfig::default();
	b_config.load_keypair(keypair_bytes).unwrap();

	let vk = read_verifying_key(&keypair_bytes[32..]).unwrap();

	/* BUILDER BENCHMARKS */
	let mut builder_group = c.benchmark_group("Builder");

	let data_1 = b"Around The World, Fatter wetter stronker" as &[u8];
	let data_2 = b"Imagine if this made sense" as &[u8];
	let data_3 = b"Fast-Acting Long-Lasting, *Bathroom Reader*" as &[u8];

	// Configure benchmark
	builder_group.throughput(Throughput::Bytes((data_1.len() + data_2.len() + data_3.len()) as u64));

	builder_group.bench_function("Builder::dump(---)", |b| {
		b.iter(|| {
			let mut builder = Builder::new();

			// Add data
			builder
				.add_leaf(Leaf::new(data_1, "d1").compress(CompressMode::Always))
				.unwrap();
			builder
				.add_leaf(Leaf::new(data_2, "d2").compress(CompressMode::Never))
				.unwrap();
			builder
				.add_leaf(Leaf::new(data_3, "d3").compress(CompressMode::Detect))
				.unwrap();

			// Dump data
			builder.dump(Sink::new(), &b_config).unwrap();
		});
	});

	// Drop Builder group
	drop(builder_group);

	/* ARCHIVE BENCHMARKS */
	let mut throughput_group = c.benchmark_group("Loader");
	let mut target = io::Cursor::new(vec![]);

	{
		// Builds an archive source from which to benchmark
		let mut builder = Builder::new().template(Leaf::default().encrypt(false).sign(false));

		// Add data
		builder.add(data_1, "d1").unwrap();
		builder.add(data_2, "d2").unwrap();
		builder.add(data_3, "d3").unwrap();

		// Dump data
		black_box(builder.dump(&mut target, &b_config).unwrap());
	}

	// Load data
	throughput_group.throughput(Throughput::Elements(3));

	let mut archive = Archive::with_key(&mut target, &vk).unwrap();

	throughput_group.bench_function("Archive::fetch(---)", |b| {
		// Load data
		b.iter(|| {
			black_box(archive.fetch("d1").unwrap());
			black_box(archive.fetch("d2").unwrap());
			black_box(archive.fetch("d3").unwrap());
		});
	});

	throughput_group.bench_function("Archive::fetch_mut(---)", |b| {
		// Load data
		b.iter(|| {
			black_box(archive.fetch_mut("d1").unwrap());
			black_box(archive.fetch_mut("d2").unwrap());
			black_box(archive.fetch_mut("d3").unwrap());
		});
	});

	drop(throughput_group);

	c.bench_function("Archive::LOAD_NEW", |b| {
		// How fast it takes to load a new archive
		b.iter(|| {
			black_box(Archive::with_key(&mut target, &vk).unwrap());
		})
	});
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
