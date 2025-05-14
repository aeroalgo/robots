using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200017E RID: 382
	[HandlerCategory("vvAverages"), HandlerName("JFatl")]
	public class JFatl : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000C13 RID: 3091 RVA: 0x000348B8 File Offset: 0x00032AB8
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("jfatl", new string[]
			{
				this.Length.ToString(),
				this.Phase.ToString(),
				src.GetHashCode().ToString()
			}, () => this.GenjFATL(src, this.Length, this.Phase));
		}

		// Token: 0x06000C12 RID: 3090 RVA: 0x000347D8 File Offset: 0x000329D8
		public IList<double> GenjFATL(IList<double> src, int length, int phase)
		{
			if (src.Count < 39)
			{
				return null;
			}
			double[] array = new double[src.Count];
			new JMA();
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
			return JMA.GenJMA(array, length, phase);
		}

		// Token: 0x170003F6 RID: 1014
		public IContext Context
		{
			// Token: 0x06000C14 RID: 3092 RVA: 0x00034936 File Offset: 0x00032B36
			get;
			// Token: 0x06000C15 RID: 3093 RVA: 0x0003493E File Offset: 0x00032B3E
			set;
		}

		// Token: 0x170003F4 RID: 1012
		[HandlerParameter(true, "5", Min = "1", Max = "50", Step = "1")]
		public int Length
		{
			// Token: 0x06000C0E RID: 3086 RVA: 0x00034679 File Offset: 0x00032879
			get;
			// Token: 0x06000C0F RID: 3087 RVA: 0x00034681 File Offset: 0x00032881
			set;
		}

		// Token: 0x170003F5 RID: 1013
		[HandlerParameter(true, "100", Min = "-100", Max = "100", Step = "10")]
		public int Phase
		{
			// Token: 0x06000C10 RID: 3088 RVA: 0x0003468A File Offset: 0x0003288A
			get;
			// Token: 0x06000C11 RID: 3089 RVA: 0x00034692 File Offset: 0x00032892
			set;
		}
	}
}
