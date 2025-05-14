using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200016E RID: 366
	[HandlerCategory("vvAverages"), HandlerName("FTLM")]
	public class FTLM : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000B96 RID: 2966 RVA: 0x00030080 File Offset: 0x0002E280
		public IList<double> Execute(IList<double> src)
		{
			if (src.Count < 44)
			{
				return null;
			}
			double[] array = new double[src.Count];
			double[] array2 = new double[]
			{
				0.436040945,
				0.3658689069,
				0.2460452079,
				0.1104506886,
				-0.0054034585,
				-0.0760367731,
				-0.0933058722,
				-0.0670110374,
				-0.0190795053,
				0.0259609206,
				0.0502044896,
				0.0477818607,
				0.0249252327,
				-0.0047706151,
				-0.0272432537,
				-0.0338917071,
				-0.0244141482,
				-0.0055774838,
				0.0128149838,
				0.0226522218,
				0.0208778257,
				0.0100299086,
				-0.0036771622,
				-0.013674485,
				-0.0160483392,
				-0.0108597376,
				-0.0016060704,
				0.0069480557,
				0.0110573605,
				0.0095711419,
				0.0040444064,
				-0.0023824623,
				-0.0067093714,
				-0.00720034,
				-0.004771771,
				0.0005541115,
				0.000786016,
				0.0130129076,
				0.0040364019
			};
			double[] array3 = new double[]
			{
				-0.0025097319,
				0.0513007762,
				0.1142800493,
				0.169934286,
				0.2025269304,
				0.2025269304,
				0.169934286,
				0.1142800493,
				0.0513007762,
				-0.0025097319,
				-0.0353166244,
				-0.0433375629,
				-0.0311244617,
				-0.0088618137,
				0.0120580088,
				0.0233183633,
				0.0221931304,
				0.0115769653,
				-0.0022157966,
				-0.0126536111,
				-0.0157416029,
				-0.011339583,
				-0.002590561,
				0.0059521459,
				0.0105212252,
				0.0096970755,
				0.0046585685,
				-0.001707923,
				-0.0063513565,
				-0.007453935,
				-0.0050439973,
				-0.0007459678,
				0.0032271474,
				0.0051357867,
				0.0044454862,
				0.0018784961,
				-0.0011065767,
				-0.0031162862,
				-0.0033443253,
				-0.0022163335,
				0.0002573669,
				0.000365079,
				0.0060440751,
				0.0018747783
			};
			for (int i = 0; i < 44; i++)
			{
				array[i] = src[i];
			}
			for (int j = 44; j < src.Count; j++)
			{
				double num = 0.0;
				double num2 = 0.0;
				for (int k = 0; k < 44; k++)
				{
					if (k < 39)
					{
						num += array2[k] * src[j - k];
					}
					num2 += array3[k] * src[j - k];
				}
				array[j] = num - num2;
			}
			return array;
		}

		// Token: 0x170003D1 RID: 977
		public IContext Context
		{
			// Token: 0x06000B97 RID: 2967 RVA: 0x00030161 File Offset: 0x0002E361
			get;
			// Token: 0x06000B98 RID: 2968 RVA: 0x00030169 File Offset: 0x0002E369
			set;
		}
	}
}
