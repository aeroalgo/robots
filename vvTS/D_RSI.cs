using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200012C RID: 300
	[HandlerCategory("vvRSI"), HandlerName("D_RSI")]
	public class D_RSI : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060008BE RID: 2238 RVA: 0x000252A6 File Offset: 0x000234A6
		public IList<double> Execute(IList<double> src)
		{
			return D_RSI.GenD_RSI(src, this.RSIperiod);
		}

		// Token: 0x060008BD RID: 2237 RVA: 0x00024ACC File Offset: 0x00022CCC
		public static IList<double> GenD_RSI(IList<double> src, int period)
		{
			IList<double> list = RSI.RSI_TSLab(src, period);
			for (int i = 64; i < src.Count; i++)
			{
				list[i] = 0.433154403793 * list[i] + 0.364936248838 * list[i - 1] + 0.2483161287091 * list[i - 2] + 0.1159127094262 * list[i - 3] + 0.001920177025782 * list[i - 4] - 0.069061915935 * list[i - 5] - 0.0890366343384 * list[i - 6] - 0.0670352313955 * list[i - 7] - 0.02354570920082 * list[i - 8] + 0.01860246173456 * list[i - 9] + 0.042676156901 * list[i - 10] + 0.0429876844454 * list[i - 11] + 0.02477765082083 * list[i - 12] - 0.0001576627378414 * list[i - 13] - 0.01967662445848 * list[i - 14] - 0.02646547548004 * list[i - 15] - 0.02024560032283 * list[i - 16] - 0.00623702818584 * list[i - 17] + 0.00780018663194 * list[i - 18] + 0.01584087110996 * list[i - 19] + 0.01529850741755 * list[i - 20] + 0.00776771039355 * list[i - 21] - 0.002822554123202 * list[i - 22] - 0.01184238041638 * list[i - 23] - 0.01459330298226 * list[i - 24] - 0.00702238938753 * list[i - 25] + 0.01182599304835 * list[i - 26] + 0.02754073932338 * list[i - 27] - 0.02827477612793 * list[i - 28] + 0.0066596554738 * list[i - 29];
				list[i] = 0.462339155006 * list[i] + 0.380746443532 * list[i - 1] + 0.243591169064 * list[i - 2] + 0.0930418466471 * list[i - 3] - 0.02835684040755 * list[i - 4] - 0.0928288862464 * list[i - 5] - 0.095652836168 * list[i - 6] - 0.0539388169505 * list[i - 7] + 0.002654987501491 * list[i - 8] + 0.0454612622799 * list[i - 9] + 0.0580001102616 * list[i - 10] + 0.0402785468641 * list[i - 11] + 0.00614866359189 * list[i - 12] - 0.02531425814679 * list[i - 13] - 0.0395272802058 * list[i - 14] - 0.032128140471 * list[i - 15] - 0.00999834894376 * list[i - 16] + 0.01411758199119 * list[i - 17] + 0.02799907845255 * list[i - 18] + 0.02616839245397 * list[i - 19] + 0.01148168026699 * list[i - 20] - 0.00716063840872 * list[i - 21] - 0.02000122316853 * list[i - 22] - 0.02129591639566 * list[i - 23] - 0.01171727150172 * list[i - 24] + 0.002714077142857 * list[i - 25] + 0.01417503605156 * list[i - 26] + 0.01719491293035 * list[i - 27] + 0.0111562843653 * list[i - 28] + 0.0001091928592001 * list[i - 29] - 0.00982781479638 * list[i - 30] - 0.01370579701559 * list[i - 31] - 0.01021209353832 * list[i - 32] - 0.001840552264328 * list[i - 33] + 0.00662015497002 * list[i - 34] + 0.01084278724001 * list[i - 35] + 0.00907911490258 * list[i - 36] + 0.002807403486449 * list[i - 37] - 0.00430091936446 * list[i - 38] - 0.00853651829218 * list[i - 39] - 0.00797780175591 * list[i - 40] - 0.00331517566666 * list[i - 41] + 0.002697670688571 * list[i - 42] + 0.00685111113574 * list[i - 43] + 0.00709078390938 * list[i - 44] + 0.00353516270592 * list[i - 45] - 0.001722941012513 * list[i - 46] - 0.00584447134884 * list[i - 47] - 0.00662609869769 * list[i - 48] - 0.00365461380634 * list[i - 49] + 0.001483708889269 * list[i - 50] + 0.00592939018204 * list[i - 51] + 0.00693617932732 * list[i - 52] + 0.00340198228418 * list[i - 53] - 0.003034976534982 * list[i - 54] - 0.00826283181286 * list[i - 55] - 0.0076635448456 * list[i - 56] + 0.000368305995778 * list[i - 57] + 0.01089806279931 * list[i - 58] + 0.01226998232534 * list[i - 59] - 0.00558538690631 * list[i - 60] - 0.02808106424832 * list[i - 61] + 0.02711041260296 * list[i - 62] - 0.00718757578535 * list[i - 63];
			}
			return list;
		}

		// Token: 0x170002CC RID: 716
		public IContext Context
		{
			// Token: 0x060008BF RID: 2239 RVA: 0x000252B4 File Offset: 0x000234B4
			get;
			// Token: 0x060008C0 RID: 2240 RVA: 0x000252BC File Offset: 0x000234BC
			set;
		}

		// Token: 0x170002CB RID: 715
		[HandlerParameter(true, "14", Min = "2", Max = "30", Step = "0")]
		public int RSIperiod
		{
			// Token: 0x060008BB RID: 2235 RVA: 0x00024AB9 File Offset: 0x00022CB9
			get;
			// Token: 0x060008BC RID: 2236 RVA: 0x00024AC1 File Offset: 0x00022CC1
			set;
		}
	}
}
