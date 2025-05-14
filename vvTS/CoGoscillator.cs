using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200001A RID: 26
	[HandlerCategory("vvIndicators"), HandlerName("Center of Gravity")]
	public class CoGoscillator : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060000D9 RID: 217 RVA: 0x00004BEF File Offset: 0x00002DEF
		public IList<double> Execute(IList<double> src)
		{
			return CoGoscillator.GenCOG(src, this.Context, this.CoG_Period);
		}

		// Token: 0x060000D8 RID: 216 RVA: 0x00004B48 File Offset: 0x00002D48
		public static IList<double> GenCOG(IList<double> src, IContext context, int cogperiod)
		{
			double[] array = new double[src.Count];
			double num = 0.0;
			double num2 = 0.0;
			for (int i = cogperiod; i < src.Count; i++)
			{
				for (int j = 0; j < cogperiod; j++)
				{
					num += (double)(1 + j) * src[i - j];
					num2 += src[i - j];
				}
				if (num2 != 0.0)
				{
					double num3 = -num / num2 + (double)((cogperiod + 1) / 2);
					array[i] = -(num3 * 10000.0) - 4997.0;
				}
			}
			return array;
		}

		// Token: 0x17000046 RID: 70
		[HandlerParameter(true, "14", Min = "3", Max = "20", Step = "1")]
		public int CoG_Period
		{
			// Token: 0x060000D6 RID: 214 RVA: 0x00004B34 File Offset: 0x00002D34
			get;
			// Token: 0x060000D7 RID: 215 RVA: 0x00004B3C File Offset: 0x00002D3C
			set;
		}

		// Token: 0x17000047 RID: 71
		public IContext Context
		{
			// Token: 0x060000DA RID: 218 RVA: 0x00004C03 File Offset: 0x00002E03
			get;
			// Token: 0x060000DB RID: 219 RVA: 0x00004C0B File Offset: 0x00002E0B
			set;
		}
	}
}
