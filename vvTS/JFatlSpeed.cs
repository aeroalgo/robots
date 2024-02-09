using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000026 RID: 38
	[HandlerCategory("vvIndicators"), HandlerName("JFatlSpeed")]
	public class JFatlSpeed : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000163 RID: 355 RVA: 0x0000690B File Offset: 0x00004B0B
		public IList<double> Execute(IList<double> src)
		{
			return this.GenJFatlSpeed(src, this.Length, this.Smooth, this.MOMPeriod);
		}

		// Token: 0x06000162 RID: 354 RVA: 0x00006848 File Offset: 0x00004A48
		public IList<double> GenJFatlSpeed(IList<double> src, int _length, int _smooth, int _momperiod)
		{
			if (src.Count < 39)
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
			for (int i = 0; i < 39; i++)
			{
				array[i] = src[38];
			}
			for (int j = 39; j < src.Count; j++)
			{
				for (int k = 0; k < 39; k++)
				{
					array[j] += array2[k] * src[j - k];
				}
			}
			IList<double> src2 = JMA.GenJMA(array, _length, 100);
			IList<double> src3 = Momentum.GenMomentum(src2, _momperiod, 0, 0);
			return JMA.GenJMA(src3, _smooth, 100);
		}

		// Token: 0x17000077 RID: 119
		public IContext Context
		{
			// Token: 0x06000164 RID: 356 RVA: 0x00006926 File Offset: 0x00004B26
			get;
			// Token: 0x06000165 RID: 357 RVA: 0x0000692E File Offset: 0x00004B2E
			set;
		}

		// Token: 0x17000075 RID: 117
		[HandlerParameter(true, "8", Min = "1", Max = "20", Step = "1")]
		public int Length
		{
			// Token: 0x0600015E RID: 350 RVA: 0x000066EC File Offset: 0x000048EC
			get;
			// Token: 0x0600015F RID: 351 RVA: 0x000066F4 File Offset: 0x000048F4
			set;
		}

		// Token: 0x17000074 RID: 116
		[HandlerParameter(true, "1", Min = "1", Max = "50", Step = "1")]
		public int MOMPeriod
		{
			// Token: 0x0600015C RID: 348 RVA: 0x000066DB File Offset: 0x000048DB
			get;
			// Token: 0x0600015D RID: 349 RVA: 0x000066E3 File Offset: 0x000048E3
			set;
		}

		// Token: 0x17000076 RID: 118
		[HandlerParameter(true, "2", Min = "1", Max = "20", Step = "1")]
		public int Smooth
		{
			// Token: 0x06000160 RID: 352 RVA: 0x000066FD File Offset: 0x000048FD
			get;
			// Token: 0x06000161 RID: 353 RVA: 0x00006705 File Offset: 0x00004905
			set;
		}
	}
}
