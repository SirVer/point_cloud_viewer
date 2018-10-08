#include <cinttypes>
#include <cstdlib>
#include <iostream>

#include "draco/compression/encode.h"
#include "draco/io/point_cloud_io.h"
#include "draco/attributes/attribute_quantization_transform.h"

// NOCOM(#sirver): should be compatible to Point in src/lib.rs
struct Point {
	std::array<float, 3> position;
	std::array<uint8_t, 4> color;
	float intensity;
};
static_assert(sizeof(Point) == 3*4+4+4, "Point is weirdly packed.");

enum Result {
	SUCCESS = 0,
	IO_ERR = 1,
	ENCODE_ERR = 2,
};

extern "C" {

int32_t encode_points(const Point *points, uint32_t num_points,
		uint32_t min_bits,
                  unsigned char *output_buffer, uint32_t *output_len) {
  // std::cerr << "num_points: " << num_points << std::endl;

  // std::cerr << "ALIVE 1" << std::endl;
  draco::PointCloud pc;
  pc.set_num_points(num_points);
  int pos_att_id;
  {
    draco::GeometryAttribute va;
    va.Init(draco::GeometryAttribute::POSITION, nullptr, 3, draco::DT_FLOAT32,
            false, sizeof(float) * 3, 0);
    pos_att_id = pc.AddAttribute(va, true, num_points);
  }
  // std::cerr << "pos_att_id: " << pos_att_id << std::endl;
  // std::cerr << "ALIVE 2" << std::endl;

  int clr_att_id;
  {
	 draco::GeometryAttribute va;
    // using UINT_8 crashes here. So we stick to floats all the way.
	 va.Init(draco::GeometryAttribute::COLOR, nullptr, 4, draco::DT_UINT8,
				  false, sizeof(uint8_t) * 4, 0);
		clr_att_id = pc.AddAttribute(va, true, num_points);
  }
  // std::cerr << "clr_att_id: " << clr_att_id << std::endl;

  // std::cerr << "ALIVE 3" << std::endl;
  int intensity_att_id;
  {
    draco::GeometryAttribute va;
    va.Init(draco::GeometryAttribute::GENERIC, nullptr, 1, draco::DT_FLOAT32,
            /*normalized=*/false, sizeof(float), 0);
    intensity_att_id = pc.AddAttribute(va, true, num_points);
  }

  // NOCOM(#sirver): look into fast memory copy path in PointCloudBuilder.
  // std::cerr << "ALIVE 4" << std::endl;
  for (draco::PointIndex i(0); i < num_points; ++i) {
	  {
		  draco::PointAttribute *att = pc.attribute(pos_att_id);
		  att->SetAttributeValue(att->mapped_index(i), &points[i.value()].position[0]);
	  }
	  {
		  draco::PointAttribute *att = pc.attribute(clr_att_id);
		  att->SetAttributeValue(att->mapped_index(i), &points[i.value()].color[0]);
	  }
	  {
		  draco::PointAttribute *att = pc.attribute(intensity_att_id);
		  att->SetAttributeValue(att->mapped_index(i), &points[i.value()].intensity);
	  }
  }

    // pc.DeduplicateAttributeValues();
    // pc.DeduplicatePointIds();
  
  // std::cerr << "SumRed: " << (sum_red / num_points) << std::endl;
  //

  // std::cerr << "ALIVE 5" << std::endl;
  draco::EncoderBuffer buffer;

  draco::Encoder encoder;
  encoder.SetSpeedOptions(0, 0);

  // NOCOM(#sirver): what else is interesting?
  // // Setup encoder options.
  // if (options.pos_quantization_bits > 0) {
  encoder.SetAttributeQuantization(draco::GeometryAttribute::POSITION, min_bits);
  encoder.SetAttributeQuantization(draco::GeometryAttribute::COLOR, 8);
  encoder.SetAttributeQuantization(draco::GeometryAttribute::GENERIC, 32);
  // }
  // if (options.tex_coords_quantization_bits > 0) {
  // encoder.SetAttributeQuantization(draco::GeometryAttribute::TEX_COORD,
  // options.tex_coords_quantization_bits);
  // }
  // std::cerr << "ALIVE 6" << std::endl;
  // if (options.normals_quantization_bits > 0) {
  // encoder.SetAttributeQuantization(draco::GeometryAttribute::NORMAL,
  // options.normals_quantization_bits);
  // }
  // if (options.generic_quantization_bits > 0) {
  // encoder.SetAttributeQuantization(draco::GeometryAttribute::GENERIC,
  // options.generic_quantization_bits);
  // }
  // encoder.SetSpeedOptions(speed, speed);
  const draco::Status status = encoder.EncodePointCloudToBuffer(pc, &buffer);
  // std::cerr << "ALIVE 7" << std::endl;
  if (!status.ok()) {
	  std::cerr << "status: '" << status << "'" << std::endl;
    return ENCODE_ERR;
  }
  // std::cerr << "ALIVE 8" << std::endl;

  // std::cerr << "buffer.size(): " << buffer.size() << std::endl;
  memcpy(output_buffer, buffer.data(), buffer.size());
  *output_len = buffer.size();

  decode the output here just to see that it works.


  // std::cerr << "ALIVE 9" << std::endl;
  return SUCCESS;
}

  int GetQuantizationBitsFromAttribute(const draco::PointAttribute *att) {
    if (att == nullptr)
      return -1;
    draco::AttributeQuantizationTransform transform;
    if (!transform.InitFromAttribute(*att))
      return -1;
    return transform.quantization_bits();
  }

int decode_points(const char *buffer, uint32_t len, uint32_t expected_num_points,
		Point *points) {
	draco::DecoderBuffer decoder_buffer;
	decoder_buffer.Init(buffer, len);

	draco::Decoder decoder;
	decoder.options()->SetAttributeInt(draco::GeometryAttribute::POSITION, "quantization_bits", 10);
	decoder.options()->SetAttributeInt(draco::GeometryAttribute::COLOR, "quantization_bits", 8);
	decoder.options()->SetAttributeInt(draco::GeometryAttribute::GENERIC, "quantization_bits", 32);
  // decoder.SetAttributeQuantization(draco::GeometryAttribute::COLOR, 32);
  // decoder.SetAttributeQuantization(draco::GeometryAttribute::GENERIC, 32);

	auto status_or = decoder.DecodePointCloudFromBuffer(&decoder_buffer);
	if (!status_or.ok()) {
	  std::cerr << "status_or: '" << status_or.status() << "'" << std::endl;
	  return ENCODE_ERR;
	}

	auto pc = std::move(status_or).value();
	if (pc->num_points() != expected_num_points) {
		return ENCODE_ERR;
	}

	auto* pos_att_id = pc->GetNamedAttribute(draco::GeometryAttribute::POSITION);
	auto* color_att_id = pc->GetNamedAttribute(draco::GeometryAttribute::COLOR);
	auto* intensity_att_id = pc->GetNamedAttribute(draco::GeometryAttribute::GENERIC);

	for (draco::PointIndex i(0); i < expected_num_points; ++i) {
          pos_att_id->GetValue(pos_att_id->mapped_index(i),
                                         &points[i.value()].position);
          color_att_id->GetValue(color_att_id->mapped_index(i),
                                 &points[i.value()].color);
          intensity_att_id->GetValue(intensity_att_id->mapped_index(i),
                                     &points[i.value()].intensity);
	  }
	return SUCCESS;
}

}

