using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200016B RID: 363
	[HandlerCategory("vvAverages"), HandlerName("FATL")]
	public class FATL : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000B87 RID: 2951 RVA: 0x0002F37D File Offset: 0x0002D57D
		public IList<double> Execute(IList<double> src)
		{
			return FATL.GenFATL(src);
		}

		// Token: 0x06000B86 RID: 2950 RVA: 0x0002F2E0 File Offset: 0x0002D4E0
		public static IList<double> GenFATL(IList<double> src)
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
				array[i] = src[i];
			}
			for (int j = 39; j < src.Count; j++)
			{
				for (int k = 0; k < 39; k++)
				{
					array[j] += array2[k] * src[j - k];
				}
			}
			return array;
		}

		// Token: 0x170003CD RID: 973
		public IContext Context
		{
			// Token: 0x06000B88 RID: 2952 RVA: 0x0002F385 File Offset: 0x0002D585
			get;
			// Token: 0x06000B89 RID: 2953 RVA: 0x0002F38D File Offset: 0x0002D58D
			set;
		}
	}
}
