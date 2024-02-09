using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000165 RID: 357
	[HandlerCategory("vvAverages"), HandlerName("REMA")]
	public class REMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000B4F RID: 2895 RVA: 0x0002E436 File Offset: 0x0002C636
		public IList<double> Execute(IList<double> src)
		{
			return REMA.GenREMA(src, this.Period, this.Lambda);
		}

		// Token: 0x06000B4E RID: 2894 RVA: 0x0002E3E8 File Offset: 0x0002C5E8
		public static IList<double> GenREMA(IList<double> src, int _Period, double _Lambda)
		{
			double[] array = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				if (i < _Period)
				{
					array[i] = src[i];
				}
				else
				{
					array[i] = EMA.iREMA(src[i], array, _Period, _Lambda, i);
				}
			}
			return array;
		}

		// Token: 0x170003BA RID: 954
		public IContext Context
		{
			// Token: 0x06000B50 RID: 2896 RVA: 0x0002E44A File Offset: 0x0002C64A
			get;
			// Token: 0x06000B51 RID: 2897 RVA: 0x0002E452 File Offset: 0x0002C652
			set;
		}

		// Token: 0x170003B9 RID: 953
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "1", Step = "0.01")]
		public double Lambda
		{
			// Token: 0x06000B4C RID: 2892 RVA: 0x0002E3D7 File Offset: 0x0002C5D7
			get;
			// Token: 0x06000B4D RID: 2893 RVA: 0x0002E3DF File Offset: 0x0002C5DF
			set;
		}

		// Token: 0x170003B8 RID: 952
		[HandlerParameter(true, "15", Min = "1", Max = "30", Step = "1")]
		public int Period
		{
			// Token: 0x06000B4A RID: 2890 RVA: 0x0002E3C6 File Offset: 0x0002C5C6
			get;
			// Token: 0x06000B4B RID: 2891 RVA: 0x0002E3CE File Offset: 0x0002C5CE
			set;
		}
	}
}
