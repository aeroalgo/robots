using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000169 RID: 361
	[HandlerCategory("vvAverages"), HandlerName("EMASlope")]
	public class EMASlope : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000B77 RID: 2935 RVA: 0x0002EDCB File Offset: 0x0002CFCB
		public IList<double> Execute(ISecurity src)
		{
			return this.GenEMASlope(src, this.EMAperiod, this.BarsNumber, this.Sense, this.Context);
		}

		// Token: 0x06000B76 RID: 2934 RVA: 0x0002ECB4 File Offset: 0x0002CEB4
		public IList<double> GenEMASlope(ISecurity src, int emaperiod, int barsNumber, double sense, IContext context)
		{
			int num = Math.Max(emaperiod, barsNumber);
			double[] array = new double[src.get_Bars().Count];
			IList<double> medpr = context.GetData("mpr", new string[]
			{
				"mpr"
			}, () => vvSeries.MedianPrice(src.get_Bars()));
			IList<double> data = context.GetData("eMA", new string[]
			{
				emaperiod.ToString()
			}, () => EMA.GenEMA(medpr, emaperiod));
			for (int i = num; i < src.get_Bars().Count; i++)
			{
				double num2 = data[i];
				double num3 = data[i - barsNumber];
				double num4 = num2 - num3;
				if (num4 > sense)
				{
					array[i] = num4;
				}
				else if (num4 < -sense)
				{
					array[i] = num4;
				}
				else
				{
					array[i] = 0.0;
				}
			}
			return array;
		}

		// Token: 0x170003C6 RID: 966
		[HandlerParameter(true, "2", Min = "1", Max = "20", Step = "1")]
		public int BarsNumber
		{
			// Token: 0x06000B72 RID: 2930 RVA: 0x0002EC63 File Offset: 0x0002CE63
			get;
			// Token: 0x06000B73 RID: 2931 RVA: 0x0002EC6B File Offset: 0x0002CE6B
			set;
		}

		// Token: 0x170003C8 RID: 968
		public IContext Context
		{
			// Token: 0x06000B78 RID: 2936 RVA: 0x0002EDEC File Offset: 0x0002CFEC
			get;
			// Token: 0x06000B79 RID: 2937 RVA: 0x0002EDF4 File Offset: 0x0002CFF4
			set;
		}

		// Token: 0x170003C5 RID: 965
		[HandlerParameter(true, "15", Min = "1", Max = "50", Step = "1")]
		public int EMAperiod
		{
			// Token: 0x06000B70 RID: 2928 RVA: 0x0002EC52 File Offset: 0x0002CE52
			get;
			// Token: 0x06000B71 RID: 2929 RVA: 0x0002EC5A File Offset: 0x0002CE5A
			set;
		}

		// Token: 0x170003C7 RID: 967
		[HandlerParameter(true, "3", Min = "0", Max = "20", Step = "0.1")]
		public double Sense
		{
			// Token: 0x06000B74 RID: 2932 RVA: 0x0002EC74 File Offset: 0x0002CE74
			get;
			// Token: 0x06000B75 RID: 2933 RVA: 0x0002EC7C File Offset: 0x0002CE7C
			set;
		}
	}
}
